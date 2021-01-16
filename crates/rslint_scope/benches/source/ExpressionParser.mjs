import {
    TV,
    PropName,
    StringValue,
    IsComputedPropertyKey,
} from '../static-semantics/all.mjs';
import {
    Token, TokenPrecedence,
    isPropertyOrCall,
    isMember,
    isKeyword,
} from './tokens.mjs';
import { isLineTerminator } from './Lexer.mjs';
import { FunctionParser, FunctionKind } from './FunctionParser.mjs';
import { RegExpParser } from './RegExpParser.mjs';

export class ExpressionParser extends FunctionParser {
    // Expression :
    //   AssignmentExpression
    //   Expression `,` AssignmentExpression
    parseExpression() {
        const node = this.startNode();
        const AssignmentExpression = this.parseAssignmentExpression();
        if (this.eat(Token.COMMA)) {
            node.ExpressionList = [AssignmentExpression];
            do {
                node.ExpressionList.push(this.parseAssignmentExpression());
            } while (this.eat(Token.COMMA));
            return this.finishNode(node, 'CommaOperator');
        }
        return AssignmentExpression;
    }

    // AssignmentExpression :
    //   ConditionalExpression
    //   [+Yield] YieldExpression
    //   ArrowFunction
    //   AsyncArrowFunction
    //   LeftHandSideExpression `=` AssignmentExpression
    //   LeftHandSideExpression AssignmentOperator AssignmentExpression
    //   LeftHandSideExpression LogicalAssignmentOperator AssignmentExpression
    //
    // AssignmentOperator : one of
    //   *= /= %= += -= <<= >>= >>>= &= ^= |= **=
    //
    // LogicalAssignmentOperator : one of
    //   &&= ||= ??=
    parseAssignmentExpression() {
        if (this.test(Token.YIELD) && this.scope.hasYield()) {
            return this.parseYieldExpression();
        }
        const node = this.startNode();

        this.scope.pushAssignmentInfo('assign');
        const left = this.parseConditionalExpression();
        const assignmentInfo = this.scope.popAssignmentInfo();

        if (left.type === 'IdentifierReference') {
            // `async` [no LineTerminator here] IdentifierReference [no LineTerminator here] `=>`
            if (left.name === 'async'
                && this.test(Token.IDENTIFIER)
                && !this.peek().hadLineTerminatorBefore
                && this.testAhead(Token.ARROW)
                && !this.peekAhead().hadLineTerminatorBefore) {
                assignmentInfo.clear();
                return this.parseArrowFunction(node, {
                    Arguments: [this.parseIdentifierReference()],
                }, FunctionKind.ASYNC);
            }
            // IdentifierReference [no LineTerminator here] `=>`
            if (this.test(Token.ARROW) && !this.peek().hadLineTerminatorBefore) {
                assignmentInfo.clear();
                return this.parseArrowFunction(node, { Arguments: [left] }, FunctionKind.NORMAL);
            }
        }

        // `async` [no LineTerminator here] Arguments [no LineTerminator here] `=>`
        if (left.type === 'CallExpression' && left.arrowInfo && this.test(Token.ARROW)
            && !this.peek().hadLineTerminatorBefore) {
            const last = left.Arguments[left.Arguments.length - 1];
            if (!left.arrowInfo.trailingComma || (last && last.type !== 'AssignmentRestElement')) {
                assignmentInfo.clear();
                return this.parseArrowFunction(node, left, FunctionKind.ASYNC);
            }
        }

        if (left.type === 'CoverParenthesizedExpressionAndArrowParameterList') {
            assignmentInfo.clear();
            return this.parseArrowFunction(node, left, FunctionKind.NORMAL);
        }

        switch (this.peek().type) {
            case Token.ASSIGN:
            case Token.ASSIGN_MUL:
            case Token.ASSIGN_DIV:
            case Token.ASSIGN_MOD:
            case Token.ASSIGN_ADD:
            case Token.ASSIGN_SUB:
            case Token.ASSIGN_SHL:
            case Token.ASSIGN_SAR:
            case Token.ASSIGN_SHR:
            case Token.ASSIGN_BIT_AND:
            case Token.ASSIGN_BIT_XOR:
            case Token.ASSIGN_BIT_OR:
            case Token.ASSIGN_EXP:
            case Token.ASSIGN_AND:
            case Token.ASSIGN_OR:
            case Token.ASSIGN_NULLISH:
                assignmentInfo.clear();
                this.validateAssignmentTarget(left);
                node.LeftHandSideExpression = left;
                node.AssignmentOperator = this.next().value;
                node.AssignmentExpression = this.parseAssignmentExpression();
                return this.finishNode(node, 'AssignmentExpression');
            default:
                return left;
        }
    }

    validateAssignmentTarget(node) {
        switch (node.type) {
            case 'IdentifierReference':
                if (this.isStrictMode() && (node.name === 'eval' || node.name === 'arguments')) {
                    break;
                }
                return;
            case 'CoverInitializedName':
                this.validateAssignmentTarget(node.IdentifierReference);
                return;
            case 'MemberExpression':
                return;
            case 'SuperProperty':
                return;
            case 'ParenthesizedExpression':
                if (node.Expression.type === 'ObjectLiteral' || node.Expression.type === 'ArrayLiteral') {
                    break;
                }
                this.validateAssignmentTarget(node.Expression);
                return;
            case 'ArrayLiteral':
                node.ElementList.forEach((p, i) => {
                    if (p.type === 'SpreadElement' && (i !== node.ElementList.length - 1 || node.hasTrailingComma)) {
                        this.raiseEarly('InvalidAssignmentTarget', p);
                    }
                    if (p.type === 'AssignmentExpression') {
                        this.validateAssignmentTarget(p.LeftHandSideExpression);
                    } else {
                        this.validateAssignmentTarget(p);
                    }
                });
                return;
            case 'ObjectLiteral':
                node.PropertyDefinitionList.forEach((p, i) => {
                    if (p.type === 'PropertyDefinition' && !p.PropertyName
                        && i !== node.PropertyDefinitionList.length - 1) {
                        this.raiseEarly('InvalidAssignmentTarget', p);
                    }
                    this.validateAssignmentTarget(p);
                });
                return;
            case 'PropertyDefinition':
                if (node.AssignmentExpression.type === 'AssignmentExpression') {
                    this.validateAssignmentTarget(node.AssignmentExpression.LeftHandSideExpression);
                } else {
                    this.validateAssignmentTarget(node.AssignmentExpression);
                }
                return;
            case 'Elision':
                return;
            case 'SpreadElement':
                if (node.AssignmentExpression.type === 'AssignmentExpression') {
                    break;
                }
                this.validateAssignmentTarget(node.AssignmentExpression);
                return;
            default:
                break;
        }
        this.raiseEarly('InvalidAssignmentTarget', node);
    }

    // YieldExpression :
    //   `yield`
    //   `yield` [no LineTerminator here] AssignmentExpression
    //   `yield` [no LineTerminator here] `*` AssignmentExpression
    parseYieldExpression() {
        if (this.scope.inParameters()) {
            this.raiseEarly('YieldInFormalParameters');
        }
        const node = this.startNode();
        this.expect(Token.YIELD);
        if (this.peek().hadLineTerminatorBefore) {
            node.hasStar = false;
            node.AssignmentExpression = null;
        } else {
            node.hasStar = this.eat(Token.MUL);
            if (node.hasStar) {
                node.AssignmentExpression = this.parseAssignmentExpression();
            } else {
                switch (this.peek().type) {
                    case Token.EOS:
                    case Token.SEMICOLON:
                    case Token.RBRACE:
                    case Token.RBRACK:
                    case Token.RPAREN:
                    case Token.COLON:
                    case Token.COMMA:
                    case Token.IN:
                        node.AssignmentExpression = null;
                        break;
                    default:
                        node.AssignmentExpression = this.parseAssignmentExpression();
                }
            }
        }
        if (this.scope.arrowInfoStack.length > 0) {
            this.scope.arrowInfoStack[this.scope.arrowInfoStack.length - 1].yieldExpressions.push(node);
        }
        return this.finishNode(node, 'YieldExpression');
    }

    // ConditionalExpression :
    //   ShortCircuitExpression
    //   ShortCircuitExpression `?` AssignmentExpression `:` AssignmentExpression
    parseConditionalExpression() {
        const node = this.startNode();
        const ShortCircuitExpression = this.parseShortCircuitExpression();
        if (this.eat(Token.CONDITIONAL)) {
            node.ShortCircuitExpression = ShortCircuitExpression;
            this.scope.with({ in: true }, () => {
                node.AssignmentExpression_a = this.parseAssignmentExpression();
            });
            this.expect(Token.COLON);
            node.AssignmentExpression_b = this.parseAssignmentExpression();
            return this.finishNode(node, 'ConditionalExpression');
        }
        return ShortCircuitExpression;
    }

    // ShortCircuitExpression :
    //   LogicalORExpression
    //   CoalesceExpression
    //
    // CoalesceExpression :
    //   CoalesceExpressionHead `??` BitwiseORExpression
    //
    // CoalesceExpressionHead :
    //   CoalesceExpression
    //   BitwiseORExpression
    parseShortCircuitExpression() {
        // Start parse at BIT_OR, right above AND/OR/NULLISH
        const expression = this.parseBinaryExpression(TokenPrecedence[Token.BIT_OR]);
        switch (this.peek().type) {
            case Token.AND:
            case Token.OR:
                // Drop into normal binary chain starting at OR
                return this.parseBinaryExpression(TokenPrecedence[Token.OR], expression);
            case Token.NULLISH: {
                let x = expression;
                while (this.eat(Token.NULLISH)) {
                    const node = this.startNode();
                    node.CoalesceExpressionHead = x;
                    node.BitwiseORExpression = this.parseBinaryExpression(TokenPrecedence[Token.BIT_OR]);
                    x = this.finishNode(node, 'CoalesceExpression');
                }
                return x;
            }
            default:
                return expression;
        }
    }

    parseBinaryExpression(precedence, x = this.parseUnaryExpression()) {
        let p = TokenPrecedence[this.peek().type];
        if (p >= precedence) {
            do {
                while (TokenPrecedence[this.peek().type] === p) {
                    const left = x;
                    if (p === TokenPrecedence[Token.EXP] && (left.type === 'UnaryExpression' || left.type === 'AwaitExpression')) {
                        return left;
                    }
                    const node = this.startNode(left);
                    if (this.peek().type === Token.IN && !this.scope.hasIn()) {
                        return left;
                    }
                    const op = this.next();
                    const right = this.parseBinaryExpression(op.type === Token.EXP ? p : p + 1);
                    let name;
                    switch (op.type) {
                        case Token.EXP:
                            name = 'ExponentiationExpression';
                            node.UpdateExpression = left;
                            node.ExponentiationExpression = right;
                            break;
                        case Token.MUL:
                        case Token.DIV:
                        case Token.MOD:
                            name = 'MultiplicativeExpression';
                            node.MultiplicativeExpression = left;
                            node.MultiplicativeOperator = op.value;
                            node.ExponentiationExpression = right;
                            break;
                        case Token.ADD:
                        case Token.SUB:
                            name = 'AdditiveExpression';
                            node.AdditiveExpression = left;
                            node.MultiplicativeExpression = right;
                            node.operator = op.value;
                            break;
                        case Token.SHL:
                        case Token.SAR:
                        case Token.SHR:
                            name = 'ShiftExpression';
                            node.ShiftExpression = left;
                            node.AdditiveExpression = right;
                            node.operator = op.value;
                            break;
                        case Token.LT:
                        case Token.GT:
                        case Token.LTE:
                        case Token.GTE:
                        case Token.INSTANCEOF:
                        case Token.IN:
                            name = 'RelationalExpression';
                            node.RelationalExpression = left;
                            node.ShiftExpression = right;
                            node.operator = op.value;
                            break;
                        case Token.EQ:
                        case Token.NE:
                        case Token.EQ_STRICT:
                        case Token.NE_STRICT:
                            name = 'EqualityExpression';
                            node.EqualityExpression = left;
                            node.RelationalExpression = right;
                            node.operator = op.value;
                            break;
                        case Token.BIT_AND:
                            name = 'BitwiseANDExpression';
                            node.A = left;
                            node.operator = op.value;
                            node.B = right;
                            break;
                        case Token.BIT_XOR:
                            name = 'BitwiseXORExpression';
                            node.A = left;
                            node.operator = op.value;
                            node.B = right;
                            break;
                        case Token.BIT_OR:
                            name = 'BitwiseORExpression';
                            node.A = left;
                            node.operator = op.value;
                            node.B = right;
                            break;
                        case Token.AND:
                            name = 'LogicalANDExpression';
                            node.LogicalANDExpression = left;
                            node.BitwiseORExpression = right;
                            break;
                        case Token.OR:
                            name = 'LogicalORExpression';
                            node.LogicalORExpression = left;
                            node.LogicalANDExpression = right;
                            break;
                        default:
                            this.unexpected(op);
                    }
                    x = this.finishNode(node, name);
                }
                p -= 1;
            } while (p >= precedence);
        }
        return x;
    }

    // UnaryExpression :
    //   UpdateExpression
    //   `delete` UnaryExpression
    //   `void` UnaryExpression
    //   `typeof` UnaryExpression
    //   `+` UnaryExpression
    //   `-` UnaryExpression
    //   `~` UnaryExpression
    //   `!` UnaryExpression
    //   [+Await] AwaitExpression
    parseUnaryExpression() {
        return this.scope.with({ in: true }, () => {
            if (this.test(Token.AWAIT) && this.scope.hasAwait()) {
                return this.parseAwaitExpression();
            }
            const node = this.startNode();
            switch (this.peek().type) {
                case Token.DELETE:
                case Token.VOID:
                case Token.TYPEOF:
                case Token.ADD:
                case Token.SUB:
                case Token.BIT_NOT:
                case Token.NOT:
                    node.operator = this.next().value;
                    node.UnaryExpression = this.parseUnaryExpression();
                    if (this.isStrictMode()
                        && node.operator === 'delete'
                        && node.UnaryExpression.type === 'IdentifierReference') {
                        this.raiseEarly('DeleteIdentifier', node.UnaryExpression);
                    }
                    return this.finishNode(node, 'UnaryExpression');
                default:
                    return this.parseUpdateExpression();
            }
        });
    }

    // AwaitExpression : `await` UnaryExpression
    parseAwaitExpression() {
        if (this.scope.inParameters()) {
            this.raiseEarly('AwaitInFormalParameters');
        }
        const node = this.startNode();
        this.expect(Token.AWAIT);
        node.UnaryExpression = this.parseUnaryExpression();
        if (this.scope.arrowInfoStack.length > 0) {
            this.scope.arrowInfoStack[this.scope.arrowInfoStack.length - 1].awaitExpressions.push(node);
        }
        if (!this.scope.hasReturn()) {
            this.state.hasTopLevelAwait = true;
        }
        return this.finishNode(node, 'AwaitExpression');
    }

    // UpdateExpression :
    //   LeftHandSideExpression
    //   LeftHandSideExpression [no LineTerminator here] `++`
    //   LeftHandSideExpression [no LineTerminator here] `--`
    //   `++` UnaryExpression
    //   `--` UnaryExpression
    parseUpdateExpression() {
        if (this.test(Token.INC) || this.test(Token.DEC)) {
            const node = this.startNode();
            node.operator = this.next().value;
            node.LeftHandSideExpression = null;
            node.UnaryExpression = this.parseUnaryExpression();
            this.validateAssignmentTarget(node.UnaryExpression);
            return this.finishNode(node, 'UpdateExpression');
        }
        const argument = this.parseLeftHandSideExpression();
        if (!this.peek().hadLineTerminatorBefore) {
            if (this.test(Token.INC) || this.test(Token.DEC)) {
                this.validateAssignmentTarget(argument);
                const node = this.startNode();
                node.operator = this.next().value;
                node.LeftHandSideExpression = argument;
                node.UnaryExpression = null;
                return this.finishNode(node, 'UpdateExpression');
            }
        }
        return argument;
    }

    // LeftHandSideExpression
    parseLeftHandSideExpression(allowCalls = true) {
        let result;
        switch (this.peek().type) {
            case Token.NEW:
                result = this.parseNewExpression();
                break;
            case Token.SUPER: {
                const node = this.startNode();
                this.next();
                if (this.test(Token.LPAREN)) {
                    if (!this.scope.hasSuperCall()) {
                        this.raiseEarly('InvalidSuperCall');
                    }
                    node.Arguments = this.parseArguments().Arguments;
                    result = this.finishNode(node, 'SuperCall');
                } else {
                    if (!this.scope.hasSuperProperty()) {
                        this.raiseEarly('InvalidSuperProperty');
                    }
                    if (this.eat(Token.LBRACK)) {
                        node.Expression = this.parseExpression();
                        this.expect(Token.RBRACK);
                        node.IdentifierName = null;
                    } else {
                        this.expect(Token.PERIOD);
                        node.Expression = null;
                        node.IdentifierName = this.parseIdentifierName();
                    }
                    result = this.finishNode(node, 'SuperProperty');
                }
                break;
            }
            case Token.IMPORT: {
                const node = this.startNode();
                this.next();
                if (this.scope.hasImportMeta() && this.eat(Token.PERIOD)) {
                    this.expect('meta');
                    result = this.finishNode(node, 'ImportMeta');
                } else {
                    if (!allowCalls) {
                        this.unexpected();
                    }
                    this.expect(Token.LPAREN);
                    node.AssignmentExpression = this.parseAssignmentExpression();
                    this.expect(Token.RPAREN);
                    result = this.finishNode(node, 'ImportCall');
                }
                break;
            }
            default:
                result = this.parsePrimaryExpression();
                break;
        }

        const check = allowCalls ? isPropertyOrCall : isMember;
        while (check(this.peek().type)) {
            const node = this.startNode(result);
            switch (this.peek().type) {
                case Token.LBRACK: {
                    this.next();
                    node.MemberExpression = result;
                    node.IdentifierName = null;
                    node.Expression = this.parseExpression();
                    result = this.finishNode(node, 'MemberExpression');
                    this.expect(Token.RBRACK);
                    break;
                }
                case Token.PERIOD:
                    this.next();
                    node.MemberExpression = result;
                    node.IdentifierName = this.parseIdentifierName();
                    node.Expression = null;
                    result = this.finishNode(node, 'MemberExpression');
                    break;
                case Token.LPAREN: {
                    // `async` [no LineTerminator here] `(`
                    const couldBeArrow = this.matches('async', this.currentToken)
                        && result.type === 'IdentifierReference'
                        && !this.peek().hadLineTerminatorBefore;
                    if (couldBeArrow) {
                        this.scope.pushArrowInfo(true);
                    }
                    const { Arguments, trailingComma } = this.parseArguments();
                    node.CallExpression = result;
                    node.Arguments = Arguments;
                    if (couldBeArrow) {
                        node.arrowInfo = this.scope.popArrowInfo();
                        node.arrowInfo.trailingComma = trailingComma;
                    }
                    result = this.finishNode(node, 'CallExpression');
                    break;
                }
                case Token.OPTIONAL:
                    node.MemberExpression = result;
                    node.OptionalChain = this.parseOptionalChain();
                    result = this.finishNode(node, 'OptionalExpression');
                    break;
                case Token.TEMPLATE:
                    node.MemberExpression = result;
                    node.TemplateLiteral = this.parseTemplateLiteral(true);
                    result = this.finishNode(node, 'TaggedTemplateExpression');
                    break;
                default:
                    this.unexpected();
            }
        }
        return result;
    }

    // OptionalChain
    parseOptionalChain() {
        this.expect(Token.OPTIONAL);
        let base = this.startNode();
        base.OptionalChain = null;
        if (this.test(Token.LPAREN)) {
            base.Arguments = this.parseArguments().Arguments;
        } else if (this.eat(Token.LBRACK)) {
            base.Expression = this.parseExpression();
            this.expect(Token.RBRACK);
        } else if (this.test(Token.TEMPLATE)) {
            this.raise('TemplateInOptionalChain');
        } else {
            base.IdentifierName = this.parseIdentifierName();
        }
        base = this.finishNode(base, 'OptionalChain');

        while (true) {
            const node = this.startNode();
            if (this.test(Token.LPAREN)) {
                node.OptionalChain = base;
                node.Arguments = this.parseArguments().Arguments;
                base = this.finishNode(node, 'OptionalChain');
            } else if (this.eat(Token.LBRACK)) {
                node.OptionalChain = base;
                node.Expression = this.parseExpression();
                this.expect(Token.RBRACK);
                base = this.finishNode(node, 'OptionalChain');
            } else if (this.test(Token.TEMPLATE)) {
                this.raise('TemplateInOptionalChain');
            } else if (this.eat(Token.PERIOD)) {
                node.OptionalChain = base;
                node.IdentifierName = this.parseIdentifierName();
                base = this.finishNode(node, 'OptionalChain');
            } else {
                return base;
            }
        }
    }

    // NewExpression
    parseNewExpression() {
        const node = this.startNode();
        this.expect(Token.NEW);
        if (this.scope.hasNewTarget() && this.eat(Token.PERIOD)) {
            this.expect('target');
            return this.finishNode(node, 'NewTarget');
        }
        node.MemberExpression = this.parseLeftHandSideExpression(false);
        if (this.test(Token.LPAREN)) {
            node.Arguments = this.parseArguments().Arguments;
        } else {
            node.Arguments = null;
        }
        return this.finishNode(node, 'NewExpression');
    }

    // PrimaryExpression :
    //   ...
    parsePrimaryExpression() {
        switch (this.peek().type) {
            case Token.IDENTIFIER:
            case Token.ESCAPED_KEYWORD:
            case Token.YIELD:
            case Token.AWAIT:
                // `async` [no LineTerminator here] `function`
                if (this.test('async') && this.testAhead(Token.FUNCTION)
                    && !this.peekAhead().hadLineTerminatorBefore) {
                    return this.parseFunctionExpression(FunctionKind.ASYNC);
                }
                return this.parseIdentifierReference();
            case Token.THIS: {
                const node = this.startNode();
                this.next();
                return this.finishNode(node, 'ThisExpression');
            }
            case Token.NUMBER:
            case Token.BIGINT:
                return this.parseNumericLiteral();
            case Token.STRING:
                return this.parseStringLiteral();
            case Token.NULL: {
                const node = this.startNode();
                this.next();
                return this.finishNode(node, 'NullLiteral');
            }
            case Token.TRUE:
            case Token.FALSE:
                return this.parseBooleanLiteral();
            case Token.LBRACK:
                return this.parseArrayLiteral();
            case Token.LBRACE:
                return this.parseObjectLiteral();
            case Token.FUNCTION:
                return this.parseFunctionExpression(FunctionKind.NORMAL);
            case Token.CLASS:
                return this.parseClassExpression();
            case Token.TEMPLATE:
                return this.parseTemplateLiteral();
            case Token.DIV:
            case Token.ASSIGN_DIV:
                return this.parseRegularExpressionLiteral();
            case Token.LPAREN:
                return this.parseCoverParenthesizedExpressionAndArrowParameterList();
            default:
                return this.unexpected();
        }
    }

    // NumericLiteral
    parseNumericLiteral() {
        const node = this.startNode();
        if (!this.test(Token.NUMBER) && !this.test(Token.BIGINT)) {
            this.unexpected();
        }
        node.value = this.next().value;
        return this.finishNode(node, 'NumericLiteral');
    }

    // StringLiteral
    parseStringLiteral() {
        const node = this.startNode();
        if (!this.test(Token.STRING)) {
            this.unexpected();
        }
        node.value = this.next().value;
        return this.finishNode(node, 'StringLiteral');
    }

    // BooleanLiteral :
    //   `true`
    //   `false`
    parseBooleanLiteral() {
        const node = this.startNode();
        switch (this.peek().type) {
            case Token.TRUE:
                this.next();
                node.value = true;
                break;
            case Token.FALSE:
                this.next();
                node.value = false;
                break;
            default:
                this.unexpected();
        }
        return this.finishNode(node, 'BooleanLiteral');
    }

    // ArrayLiteral :
    //   `[` `]`
    //   `[` Elision `]`
    //   `[` ElementList `]`
    //   `[` ElementList `,` `]`
    //   `[` ElementList `,` Elision `]`
    parseArrayLiteral() {
        const node = this.startNode();
        this.expect(Token.LBRACK);
        node.ElementList = [];
        node.hasTrailingComma = false;
        while (true) {
            while (this.test(Token.COMMA)) {
                const elision = this.startNode();
                this.next();
                node.ElementList.push(this.finishNode(elision, 'Elision'));
            }
            if (this.eat(Token.RBRACK)) {
                break;
            }
            if (this.test(Token.ELLIPSIS)) {
                const spread = this.startNode();
                this.next();
                spread.AssignmentExpression = this.parseAssignmentExpression();
                node.ElementList.push(this.finishNode(spread, 'SpreadElement'));
            } else {
                node.ElementList.push(this.parseAssignmentExpression());
            }
            if (this.eat(Token.RBRACK)) {
                node.hasTrailingComma = false;
                break;
            }
            node.hasTrailingComma = true;
            this.expect(Token.COMMA);
        }
        return this.finishNode(node, 'ArrayLiteral');
    }

    // ObjectLiteral :
    //   `{` `}`
    //   `{` PropertyDefinitionList `}`
    //   `{` PropertyDefinitionList `,` `}`
    parseObjectLiteral() {
        const node = this.startNode();
        this.expect(Token.LBRACE);
        node.PropertyDefinitionList = [];
        let hasProto = false;
        while (true) {
            if (this.eat(Token.RBRACE)) {
                break;
            }
            const PropertyDefinition = this.parsePropertyDefinition();
            if (!this.state.json
                && PropertyDefinition.type === 'PropertyDefinition'
                && PropertyDefinition.PropertyName
                && !IsComputedPropertyKey(PropertyDefinition.PropertyName)
                && PropertyDefinition.PropertyName.type !== 'NumericLiteral'
                && StringValue(PropertyDefinition.PropertyName).stringValue() === '__proto__') {
                if (hasProto) {
                    this.scope.registerObjectLiteralEarlyError(this.raiseEarly('DuplicateProto', PropertyDefinition.PropertyName));
                } else {
                    hasProto = true;
                }
            }
            node.PropertyDefinitionList.push(PropertyDefinition);
            if (this.eat(Token.RBRACE)) {
                break;
            }
            this.expect(Token.COMMA);
        }
        return this.finishNode(node, 'ObjectLiteral');
    }

    parsePropertyDefinition() {
        return this.parseBracketedDefinition('property');
    }

    parseFunctionExpression(kind) {
        return this.parseFunction(true, kind);
    }

    parseArguments() {
        this.expect(Token.LPAREN);
        if (this.eat(Token.RPAREN)) {
            return { Arguments: [], trailingComma: false };
        }
        const Arguments = [];
        let trailingComma = false;
        while (true) {
            const node = this.startNode();
            if (this.eat(Token.ELLIPSIS)) {
                node.AssignmentExpression = this.parseAssignmentExpression();
                Arguments.push(this.finishNode(node, 'AssignmentRestElement'));
            } else {
                Arguments.push(this.parseAssignmentExpression());
            }
            if (this.eat(Token.RPAREN)) {
                break;
            }
            this.expect(Token.COMMA);
            if (this.eat(Token.RPAREN)) {
                trailingComma = true;
                break;
            }
        }
        return { Arguments, trailingComma };
    }

    // #sec-class-definitions
    // ClassDeclaration :
    //   `class` BindingIdentifier ClassTail
    //   [+Default] `class` ClassTail
    //
    // ClassExpression :
    //   `class` BindingIdentifier? ClassTail
    parseClass(isExpression) {
        const node = this.startNode();

        this.expect(Token.CLASS);

        this.scope.with({ strict: true }, () => {
            if (!this.test(Token.LBRACE) && !this.test(Token.EXTENDS)) {
                node.BindingIdentifier = this.parseBindingIdentifier();
                if (!isExpression) {
                    this.scope.declare(node.BindingIdentifier, 'lexical');
                }
            } else if (isExpression === false && !this.scope.isDefault()) {
                this.raise('ClassMissingBindingIdentifier');
            } else {
                node.BindingIdentifier = null;
            }
            node.ClassTail = this.scope.with({ default: false }, () => this.parseClassTail());
        });

        return this.finishNode(node, isExpression ? 'ClassExpression' : 'ClassDeclaration');
    }

    // ClassTail : ClassHeritage? `{` ClassBody? `}`
    // ClassHeritage : `extends` LeftHandSideExpression
    // ClassBody : ClassElementList
    parseClassTail() {
        const node = this.startNode();

        if (this.eat(Token.EXTENDS)) {
            node.ClassHeritage = this.parseLeftHandSideExpression();
        } else {
            node.ClassHeritage = null;
        }

        this.expect(Token.LBRACE);
        if (this.eat(Token.RBRACE)) {
            node.ClassBody = null;
        } else {
            this.scope.with({ superCall: !!node.ClassHeritage }, () => {
                node.ClassBody = [];
                let hasConstructor = false;
                while (!this.eat(Token.RBRACE)) {
                    const m = this.parseClassElement();
                    node.ClassBody.push(m);

                    const name = PropName(m.MethodDefinition);
                    const isActualConstructor = !m.static
                        && !!m.MethodDefinition.UniqueFormalParameters
                        && m.MethodDefinition.type === 'MethodDefinition'
                        && name === 'constructor';
                    if (isActualConstructor) {
                        if (hasConstructor) {
                            this.raiseEarly('DuplicateConstructor', m);
                        } else {
                            hasConstructor = true;
                        }
                    }
                    if ((m.static && name === 'prototype')
                        || (!m.static && !isActualConstructor && name === 'constructor')) {
                        this.raiseEarly('InvalidMethodName', m, name);
                    }
                }
            });
        }

        return this.finishNode(node, 'ClassTail');
    }

    // ClassElement :
    //   `static` MethodDefinition
    //   MethodDefinition
    parseClassElement() {
        const node = this.startNode();
        node.static = this.eat('static');
        node.MethodDefinition = this.parseMethodDefinition(node.static);
        while (this.eat(Token.SEMICOLON)) {
            // nothing
        }
        return this.finishNode(node, 'ClassElement');
    }

    parseMethodDefinition(isStatic) {
        return this.parseBracketedDefinition('method', isStatic);
    }

    parseClassExpression() {
        return this.parseClass(true);
    }

    parseTemplateLiteral(tagged = false) {
        const node = this.startNode();
        node.TemplateSpanList = [];
        node.ExpressionList = [];
        let buffer = '';
        while (true) {
            if (this.position >= this.source.length) {
                this.raise('UnterminatedTemplate', this.position);
            }
            const c = this.source[this.position];
            switch (c) {
                case '`':
                    this.position += 1;
                    node.TemplateSpanList.push(buffer);
                    this.next();
                    if (!tagged) {
                        node.TemplateSpanList.forEach((s) => {
                            if (TV(s) === undefined) {
                                this.raise('InvalidTemplateEscape');
                            }
                        });
                    }
                    return this.finishNode(node, 'TemplateLiteral');
                case '$':
                    this.position += 1;
                    if (this.source[this.position] === '{') {
                        this.position += 1;
                        node.TemplateSpanList.push(buffer);
                        buffer = '';
                        this.next();
                        node.ExpressionList.push(this.parseExpression());
                        break;
                    }
                    buffer += c;
                    break;
                default: {
                    if (c === '\\') {
                        buffer += c;
                        this.position += 1;
                    }
                    const l = this.source[this.position];
                    this.position += 1;
                    if (isLineTerminator(l)) {
                        if (l === '\r' && this.source[this.position] === '\n') {
                            this.position += 1;
                        }
                        if (l === '\u{2028}' || l === '\u{2029}') {
                            buffer += l;
                        } else {
                            buffer += '\n';
                        }
                        this.line += 1;
                        this.columnOffset = this.position;
                    } else {
                        buffer += l;
                    }
                    break;
                }
            }
        }
    }

    // RegularExpressionLiteral :
    //   `/` RegularExpressionBody `/` RegularExpressionFlags
    parseRegularExpressionLiteral() {
        const node = this.startNode();
        this.scanRegularExpressionBody();
        node.RegularExpressionBody = this.scannedValue;
        this.scanRegularExpressionFlags();
        node.RegularExpressionFlags = this.scannedValue;
        try {
            const parse = (flags) => {
                const p = new RegExpParser(node.RegularExpressionBody);
                return p.scope(flags, () => p.parsePattern());
            };
            if (node.RegularExpressionFlags.includes('u')) {
                parse({ U: true, N: true });
            } else {
                const pattern = parse({ U: false, N: false });
                if (pattern.groupSpecifiers.size > 0) {
                    parse({ U: false, N: true });
                }
            }
        } catch (e) {
            if (e instanceof SyntaxError) {
                this.raise('Raw', node.location.startIndex + e.position + 1, e.message);
            } else {
                throw e;
            }
        }
        const fakeToken = {
            endIndex: this.position - 1,
            line: this.line - 1,
            column: this.position - this.columnOffset,
        };
        this.next();
        this.currentToken = fakeToken;
        return this.finishNode(node, 'RegularExpressionLiteral');
    }

    // CoverParenthesizedExpressionAndArrowParameterList :
    //   `(` Expression `)`
    //   `(` Expression `,` `)`
    //   `(` `)`
    //   `(` `...` BindingIdentifier `)`
    //   `(` `...` BindingPattern `)`
    //   `(` Expression `,` `...` BindingIdentifier `)`
    //   `(` Expression `.` `...` BindingPattern `)`
    parseCoverParenthesizedExpressionAndArrowParameterList() {
        const node = this.startNode();
        const commaOp = this.startNode();
        this.expect(Token.LPAREN);
        if (this.test(Token.RPAREN)) {
            if (!this.testAhead(Token.ARROW) || this.peekAhead().hadLineTerminatorBefore) {
                this.unexpected();
            }
            this.next();
            node.Arguments = [];
            return this.finishNode(node, 'CoverParenthesizedExpressionAndArrowParameterList');
        }

        this.scope.pushArrowInfo();
        this.scope.pushAssignmentInfo('arrow');

        const expressions = [];
        let rparenAfterComma;
        while (true) {
            if (this.test(Token.ELLIPSIS)) {
                const inner = this.startNode();
                this.next();
                switch (this.peek().type) {
                    case Token.LBRACE:
                    case Token.LBRACK:
                        inner.BindingPattern = this.parseBindingPattern();
                        break;
                    default:
                        inner.BindingIdentifier = this.parseBindingIdentifier();
                        break;
                }
                expressions.push(this.finishNode(inner, 'BindingRestElement'));
                this.expect(Token.RPAREN);
                break;
            }
            expressions.push(this.parseAssignmentExpression());
            if (this.eat(Token.COMMA)) {
                if (this.eat(Token.RPAREN)) {
                    rparenAfterComma = this.currentToken;
                    break;
                }
            } else {
                this.expect(Token.RPAREN);
                break;
            }
        }

        const arrowInfo = this.scope.popArrowInfo();
        const assignmentInfo = this.scope.popAssignmentInfo();

        // ArrowParameters :
        //   CoverParenthesizedExpressionAndArrowParameterList
        if (this.test(Token.ARROW) && !this.peek().hadLineTerminatorBefore) {
            node.Arguments = expressions;
            node.arrowInfo = arrowInfo;
            assignmentInfo.clear();
            return this.finishNode(node, 'CoverParenthesizedExpressionAndArrowParameterList');
        }

        // ParenthesizedExpression :
        //   `(` Expression `)`
        if (expressions[expressions.length - 1].type === 'BindingRestElement') {
            this.unexpected(expressions[expressions.length - 1]);
        }
        if (rparenAfterComma) {
            this.unexpected(rparenAfterComma);
        }
        if (expressions.length === 1) {
            node.Expression = expressions[0];
        } else {
            commaOp.ExpressionList = expressions;
            node.Expression = this.finishNode(commaOp, 'CommaOperator');
        }
        return this.finishNode(node, 'ParenthesizedExpression');
    }

    // PropertyName :
    //   LiteralPropertyName
    //   ComputedPropertyName
    // LiteralPropertyName :
    //   IdentifierName
    //   StringLiteral
    //   NumericLiteral
    // ComputedPropertyName :
    //   `[` AssignmentExpression `]`
    parsePropertyName() {
        if (this.test(Token.LBRACK)) {
            const node = this.startNode();
            this.next();
            node.ComputedPropertyName = this.parseAssignmentExpression();
            this.expect(Token.RBRACK);
            return this.finishNode(node, 'PropertyName');
        }
        if (this.test(Token.STRING)) {
            return this.parseStringLiteral();
        }
        if (this.test(Token.NUMBER) || this.test(Token.BIGINT)) {
            return this.parseNumericLiteral();
        }
        return this.parseIdentifierName();
    }

    // PropertyDefinition :
    //   IdentifierReference
    //   CoverInitializedName
    //   PropertyName `:` AssignmentExpression
    //   MethodDefinition
    //   `...` AssignmentExpression
    // MethodDefinition :
    //   PropertyName `(` UniqueFormalParameters `)` `{` FunctionBody `}`
    //   GeneratorMethod
    //   AsyncMethod
    //   AsyncGeneratorMethod
    //   `get` PropertyName `(` `)` `{` FunctionBody `}`
    //   `set` PropertyName `(` PropertySetParameterList `)` `{` FunctionBody `}`
    // GeneratorMethod :
    //   `*` PropertyName `(` UniqueFormalParameters `)` `{` GeneratorBody `}`
    // AsyncMethod :
    //   `async` [no LineTerminator here] PropertyName `(` UniqueFormalParameters `)` `{` AsyncFunctionBody `}`
    // AsyncGeneratorMethod :
    //   `async` [no LineTerminator here] `*` Propertyname `(` UniqueFormalParameters `)` `{` AsyncGeneratorBody `}`
    parseBracketedDefinition(type, isStatic = false) {
        const node = this.startNode();

        if (type === 'property' && this.eat(Token.ELLIPSIS)) {
            node.PropertyName = null;
            node.AssignmentExpression = this.parseAssignmentExpression();
            return this.finishNode(node, 'PropertyDefinition');
        }

        let isGenerator = this.eat(Token.MUL);
        let isGetter = false;
        let isSetter = false;
        let isAsync = false;
        if (!isGenerator) {
            if (this.test('get')) {
                isGetter = true;
            } else if (this.test('set')) {
                isSetter = true;
            } else if (this.test('async') && !this.peekAhead().hadLineTerminatorBefore) {
                isAsync = true;
            }
        }
        const firstName = this.parsePropertyName();
        if (!isGenerator && !isGetter && !isSetter) {
            isGenerator = this.eat(Token.MUL);
        }
        const isSpecialMethod = isGenerator || ((isSetter || isGetter || isAsync) && !this.test(Token.LPAREN));

        if (!isGenerator && type === 'property') {
            if (this.eat(Token.COLON)) {
                node.PropertyName = firstName;
                node.AssignmentExpression = this.parseAssignmentExpression();
                return this.finishNode(node, 'PropertyDefinition');
            }
            if (this.scope.assignmentInfoStack.length > 0 && this.test(Token.ASSIGN)) {
                node.IdentifierReference = firstName;
                node.IdentifierReference.type = 'IdentifierReference';
                node.Initializer = this.parseInitializerOpt();
                this.finishNode(node, 'CoverInitializedName');
                this.scope.registerObjectLiteralEarlyError(this.raiseEarly('UnexpectedToken', node));
                return node;
            }

            if (!isSpecialMethod
                && firstName.type === 'IdentifierName'
                && !this.test(Token.LPAREN)
                && !isKeyword(firstName.name)) {
                firstName.type = 'IdentifierReference';
                this.validateIdentifierReference(firstName.name, firstName);
                return firstName;
            }
        }

        node.PropertyName = (isSpecialMethod && (!isGenerator || isAsync)) ? this.parsePropertyName() : firstName;

        this.scope.with({
            lexical: true,
            variable: true,
            superProperty: true,
            await: isAsync,
            yield: isGenerator,
        }, () => {
            if (isSpecialMethod && isGetter) {
                this.expect(Token.LPAREN);
                this.expect(Token.RPAREN);
                node.PropertySetParameterList = null;
                node.UniqueFormalParameters = null;
            } else if (isSpecialMethod && isSetter) {
                this.expect(Token.LPAREN);
                node.PropertySetParameterList = [this.parseFormalParameter()];
                this.expect(Token.RPAREN);
                node.UniqueFormalParameters = null;
            } else {
                node.PropertySetParameterList = null;
                node.UniqueFormalParameters = this.parseUniqueFormalParameters();
            }

            this.scope.with({
                superCall: !isSpecialMethod
                    && !isStatic
                    && (node.PropertyName.name === 'constructor' || node.PropertyName.value === 'constructor')
                    && this.scope.hasSuperCall(),
            }, () => {
                const body = this.parseFunctionBody(isAsync, isGenerator, false);
                node[`${isAsync ? 'Async' : ''}${isGenerator ? 'Generator' : 'Function'}Body`] = body;
                if (node.UniqueFormalParameters || node.PropertySetParameterList) {
                    this.validateFormalParameters(node.UniqueFormalParameters || node.PropertySetParameterList, body, true);
                }
            });
        });

        const name = `${isAsync ? 'Async' : ''}${isGenerator ? 'Generator' : ''}Method${isAsync || isGenerator ? '' : 'Definition'}`;
        return this.finishNode(node, name);
    }
}
