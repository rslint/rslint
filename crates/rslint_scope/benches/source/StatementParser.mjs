import { IsStringValidUnicode, StringValue } from '../static-semantics/all.mjs';
import { Token, isAutomaticSemicolon, isKeywordRaw } from './tokens.mjs';
import { ExpressionParser } from './ExpressionParser.mjs';
import { FunctionKind } from './FunctionParser.mjs';
import { getDeclarations } from './Scope.mjs';

export class StatementParser extends ExpressionParser {
    semicolon() {
        if (this.eat(Token.SEMICOLON)) {
            return;
        }
        if (this.peek().hadLineTerminatorBefore || isAutomaticSemicolon(this.peek().type)) {
            return;
        }
        this.unexpected();
    }

    // StatementList :
    //   StatementListItem
    //   StatementList StatementListItem
    parseStatementList(endToken, directives) {
        const statementList = [];
        const oldStrict = this.state.strict;
        const directiveData = [];
        while (!this.eat(endToken)) {
            if (directives !== undefined && this.test(Token.STRING)) {
                const token = this.peek();
                const directive = this.source.slice(token.startIndex + 1, token.endIndex - 1);
                if (directive === 'use strict') {
                    this.state.strict = true;
                    directiveData.forEach((d) => {
                        if (/\\([1-9]|0\d)/.test(d.directive)) {
                            this.raiseEarly('IllegalOctalEscape', d.token);
                        }
                    });
                }
                directives.push(directive);
                directiveData.push({ directive, token });
            } else {
                directives = undefined;
            }

            const stmt = this.parseStatementListItem();
            statementList.push(stmt);
        }

        this.state.strict = oldStrict;

        return statementList;
    }

    // StatementListItem :
    //   Statement
    //   Declaration
    //
    // Declaration :
    //   HoistableDeclaration
    //   ClassDeclaration
    //   LexicalDeclaration
    parseStatementListItem() {
        switch (this.peek().type) {
            case Token.FUNCTION:
                return this.parseHoistableDeclaration();
            case Token.CLASS:
                return this.parseClassDeclaration();
            case Token.CONST:
                return this.parseLexicalDeclaration();
            default:
                if (this.test('let')) {
                    switch (this.peekAhead().type) {
                        case Token.LBRACE:
                        case Token.LBRACK:
                        case Token.IDENTIFIER:
                        case Token.YIELD:
                        case Token.AWAIT:
                            return this.parseLexicalDeclaration();
                        default:
                            break;
                    }
                }
                if (this.test('async') && this.testAhead(Token.FUNCTION) && !this.peekAhead().hadLineTerminatorBefore) {
                    return this.parseHoistableDeclaration();
                }
                return this.parseStatement();
        }
    }

    // HoistableDeclaration :
    //   FunctionDeclaration
    //   GeneratorDeclaration
    //   AsyncFunctionDeclaration
    //   AsyncGeneratorDeclaration
    parseHoistableDeclaration() {
        switch (this.peek().type) {
            case Token.FUNCTION:
                return this.parseFunctionDeclaration(FunctionKind.NORMAL);
            default:
                if (this.test('async') && this.testAhead(Token.FUNCTION) && !this.peekAhead().hadLineTerminatorBefore) {
                    return this.parseFunctionDeclaration(FunctionKind.ASYNC);
                }
                throw new Error('unreachable');
        }
    }

    // ClassDeclaration :
    //   `class` BindingIdentifier ClassTail
    //   [+Default] `class` ClassTail
    parseClassDeclaration() {
        return this.parseClass(false);
    }

    // LexicalDeclaration : LetOrConst BindingList `;`
    parseLexicalDeclaration() {
        const node = this.startNode();
        const letOrConst = this.eat('let') || this.expect(Token.CONST);
        node.LetOrConst = letOrConst.type === Token.CONST ? 'const' : 'let';
        node.BindingList = this.parseBindingList();
        this.semicolon();

        this.scope.declare(node.BindingList, 'lexical');
        node.BindingList.forEach((b) => {
            if (node.LetOrConst === 'const' && !b.Initializer) {
                this.raiseEarly('ConstDeclarationMissingInitializer', b);
            }
        });

        return this.finishNode(node, 'LexicalDeclaration');
    }

    // BindingList :
    //   LexicalBinding
    //   BindingList `,` LexicalBinding
    //
    // LexicalBinding :
    //   BindingIdentifier Initializer?
    //   BindingPattern Initializer
    parseBindingList() {
        const bindingList = [];
        do {
            const node = this.parseBindingElement();
            node.type = 'LexicalBinding';
            bindingList.push(node);
        } while (this.eat(Token.COMMA));
        return bindingList;
    }

    // BindingElement :
    //   SingleNameBinding
    //   BindingPattern Initializer?
    // SingleNameBinding :
    //   BindingIdentifier Initializer?
    parseBindingElement() {
        const node = this.startNode();
        if (this.test(Token.LBRACE) || this.test(Token.LBRACK)) {
            node.BindingPattern = this.parseBindingPattern();
        } else {
            node.BindingIdentifier = this.parseBindingIdentifier();
        }
        node.Initializer = this.parseInitializerOpt();
        return this.finishNode(node, node.BindingPattern ? 'BindingElement' : 'SingleNameBinding');
    }

    // BindingPattern:
    //   ObjectBindingPattern
    //   ArrayBindingPattern
    parseBindingPattern() {
        switch (this.peek().type) {
            case Token.LBRACE:
                return this.parseObjectBindingPattern();
            case Token.LBRACK:
                return this.parseArrayBindingPattern();
            default:
                return this.unexpected();
        }
    }

    // ObjectBindingPattern :
    //   `{` `}`
    //   `{` BindingRestProperty `}`
    //   `{` BindingPropertyList `}`
    //   `{` BindingPropertyList `,` BindingRestProperty? `}`
    parseObjectBindingPattern() {
        const node = this.startNode();
        this.expect(Token.LBRACE);
        node.BindingPropertyList = [];
        while (!this.eat(Token.RBRACE)) {
            if (this.test(Token.ELLIPSIS)) {
                node.BindingRestProperty = this.parseBindingRestProperty();
                this.expect(Token.RBRACE);
                break;
            } else {
                node.BindingPropertyList.push(this.parseBindingProperty());
                if (!this.eat(Token.COMMA)) {
                    this.expect(Token.RBRACE);
                    break;
                }
            }
        }
        return this.finishNode(node, 'ObjectBindingPattern');
    }

    // BindingProperty :
    //   SingleNameBinding
    //   PropertyName : BindingElement
    parseBindingProperty() {
        const node = this.startNode();
        const name = this.parsePropertyName();
        if (this.eat(Token.COLON)) {
            node.PropertyName = name;
            node.BindingElement = this.parseBindingElement();
            return this.finishNode(node, 'BindingProperty');
        }
        node.BindingIdentifier = name;
        if (name.type === 'IdentifierName') {
            name.type = 'BindingIdentifier';
        } else {
            this.unexpected(name);
        }
        node.Initializer = this.parseInitializerOpt();
        return this.finishNode(node, 'SingleNameBinding');
    }

    // BindingRestProperty :
    //  `...` BindingIdentifier
    parseBindingRestProperty() {
        const node = this.startNode();
        this.expect(Token.ELLIPSIS);
        node.BindingIdentifier = this.parseBindingIdentifier();
        return this.finishNode(node, 'BindingRestProperty');
    }

    // ArrayBindingPattern :
    //   `[` Elision? BindingRestElement `]`
    //   `[` BindingElementList `]`
    //   `[` BindingElementList `,` Elision? BindingRestElement `]`
    parseArrayBindingPattern() {
        const node = this.startNode();
        this.expect(Token.LBRACK);
        node.BindingElementList = [];
        while (true) {
            while (this.test(Token.COMMA)) {
                const elision = this.startNode();
                this.next();
                node.BindingElementList.push(this.finishNode(elision, 'Elision'));
            }
            if (this.eat(Token.RBRACK)) {
                break;
            }
            if (this.test(Token.ELLIPSIS)) {
                node.BindingRestElement = this.parseBindingRestElement();
                this.expect(Token.RBRACK);
                break;
            } else {
                node.BindingElementList.push(this.parseBindingElement());
            }
            if (this.eat(Token.RBRACK)) {
                break;
            }
            this.expect(Token.COMMA);
        }
        return this.finishNode(node, 'ArrayBindingPattern');
    }

    // BindingRestElement :
    //   `...` BindingIdentifier
    //   `...` BindingPattern
    parseBindingRestElement() {
        const node = this.startNode();
        this.expect(Token.ELLIPSIS);
        switch (this.peek().type) {
            case Token.LBRACE:
            case Token.LBRACK:
                node.BindingPattern = this.parseBindingPattern();
                break;
            default:
                node.BindingIdentifier = this.parseBindingIdentifier();
                break;
        }
        return this.finishNode(node, 'BindingRestElement');
    }

    // Initializer : `=` AssignmentExpression
    parseInitializerOpt() {
        if (this.eat(Token.ASSIGN)) {
            return this.parseAssignmentExpression();
        }
        return null;
    }

    // FunctionDeclaration
    parseFunctionDeclaration(kind) {
        return this.parseFunction(false, kind);
    }

    // Statement :
    //   ...
    parseStatement() {
        switch (this.peek().type) {
            case Token.LBRACE:
                return this.parseBlockStatement();
            case Token.VAR:
                return this.parseVariableStatement();
            case Token.SEMICOLON: {
                const node = this.startNode();
                this.next();
                return this.finishNode(node, 'EmptyStatement');
            }
            case Token.IF:
                return this.parseIfStatement();
            case Token.DO:
                return this.parseDoWhileStatement();
            case Token.WHILE:
                return this.parseWhileStatement();
            case Token.FOR:
                return this.parseForStatement();
            case Token.SWITCH:
                return this.parseSwitchStatement();
            case Token.CONTINUE:
            case Token.BREAK:
                return this.parseBreakContinueStatement();
            case Token.RETURN:
                return this.parseReturnStatement();
            case Token.WITH:
                return this.parseWithStatement();
            case Token.THROW:
                return this.parseThrowStatement();
            case Token.TRY:
                return this.parseTryStatement();
            case Token.DEBUGGER:
                return this.parseDebuggerStatement();
            default:
                return this.parseExpressionStatement();
        }
    }

    // BlockStatement : Block
    parseBlockStatement() {
        return this.parseBlock();
    }

    // Block : `{` StatementList `}`
    parseBlock(lexical = true) {
        const node = this.startNode();
        this.expect(Token.LBRACE);
        this.scope.with({ lexical }, () => {
            node.StatementList = this.parseStatementList(Token.RBRACE);
        });
        return this.finishNode(node, 'Block');
    }

    // VariableStatement : `var` VariableDeclarationList `;`
    parseVariableStatement() {
        const node = this.startNode();
        this.expect(Token.VAR);
        node.VariableDeclarationList = this.parseVariableDeclarationList();
        this.semicolon();
        this.scope.declare(node.VariableDeclarationList, 'variable');
        return this.finishNode(node, 'VariableStatement');
    }

    // VariableDeclarationList :
    //   VariableDeclaration
    //   VariableDeclarationList `,` VariableDeclaration
    parseVariableDeclarationList(firstDeclarationRequiresInit = true) {
        const declarationList = [];
        do {
            const node = this.parseVariableDeclaration(firstDeclarationRequiresInit);
            declarationList.push(node);
        } while (this.eat(Token.COMMA));
        return declarationList;
    }

    // VariableDeclaration :
    //   BindingIdentifier Initializer?
    //   BindingPattern Initializer
    parseVariableDeclaration(firstDeclarationRequiresInit) {
        const node = this.startNode();
        switch (this.peek().type) {
            case Token.LBRACE:
            case Token.LBRACK:
                node.BindingPattern = this.parseBindingPattern();
                if (firstDeclarationRequiresInit) {
                    this.expect(Token.ASSIGN);
                    node.Initializer = this.parseAssignmentExpression();
                } else {
                    node.Initializer = this.parseInitializerOpt();
                }
                break;
            default:
                node.BindingIdentifier = this.parseBindingIdentifier();
                node.Initializer = this.parseInitializerOpt();
                break;
        }
        return this.finishNode(node, 'VariableDeclaration');
    }

    // IfStatement :
    //  `if` `(` Expression `)` Statement `else` Statement
    //  `if` `(` Expression `)` Statement [lookahead != `else`]
    parseIfStatement() {
        const node = this.startNode();
        this.expect(Token.IF);
        this.expect(Token.LPAREN);
        node.Expression = this.parseExpression();
        this.expect(Token.RPAREN);
        node.Statement_a = this.parseStatement();
        if (this.eat(Token.ELSE)) {
            node.Statement_b = this.parseStatement();
        }
        return this.finishNode(node, 'IfStatement');
    }

    // `while` `(` Expression `)` Statement
    parseWhileStatement() {
        const node = this.startNode();
        this.expect(Token.WHILE);
        this.expect(Token.LPAREN);
        node.Expression = this.parseExpression();
        this.expect(Token.RPAREN);
        this.scope.with({ label: 'loop' }, () => {
            node.Statement = this.parseStatement();
        });
        return this.finishNode(node, 'WhileStatement');
    }

    // `do` Statement `while` `(` Expression `)` `;`
    parseDoWhileStatement() {
        const node = this.startNode();
        this.expect(Token.DO);
        this.scope.with({ label: 'loop' }, () => {
            node.Statement = this.parseStatement();
        });
        this.expect(Token.WHILE);
        this.expect(Token.LPAREN);
        node.Expression = this.parseExpression();
        this.expect(Token.RPAREN);
        // Semicolons are completely optional after a do-while, even without a newline
        this.eat(Token.SEMICOLON);
        return this.finishNode(node, 'DoWhileStatement');
    }

    // `for` `(` [lookahead != `let` `[`] Expression? `;` Expression? `;` Expression? `)` Statement
    // `for` `(` `var` VariableDeclarationList `;` Expression? `;` Expression? `)` Statement
    // `for` `(` LexicalDeclaration Expression? `;` Expression? `)` Statement
    // `for` `(` [lookahead != `let` `[`] LeftHandSideExpression `in` Expression `)` Statement
    // `for` `(` `var` ForBinding `in` Expression `)` Statement
    // `for` `(` ForDeclaration `in` Expression `)` Statement
    // `for` `(` [lookahead != `let`] LeftHandSideExpression `of` AssignmentExpression `)` Statement
    // `for` `(` `var` ForBinding `of` AssignmentExpression `)` Statement
    // `for` `(` ForDeclaration `of` AssignmentExpression `)` Statement
    // `for` `await` `(` [lookahead != `let`] LeftHandSideExpression `of` AssignmentExpression `)` Statement
    // `for` `await` `(` `var` ForBinding `of` AssignmentExpression `)` Statement
    // `for` `await` `(` ForDeclaration `of` AssignmentExpression `)` Statement
    //
    // ForDeclaration : LetOrConst ForBinding
    parseForStatement() {
        return this.scope.with({
            lexical: true,
            label: 'loop',
        }, () => {
            const node = this.startNode();
            this.expect(Token.FOR);
            const isAwait = this.scope.hasAwait() && this.eat(Token.AWAIT);
            if (isAwait && !this.scope.hasReturn()) {
                this.state.hasTopLevelAwait = true;
            }
            this.expect(Token.LPAREN);
            if (isAwait && this.test(Token.SEMICOLON)) {
                this.unexpected();
            }
            if (this.eat(Token.SEMICOLON)) {
                if (!this.test(Token.SEMICOLON)) {
                    node.Expression_b = this.parseExpression();
                }
                this.expect(Token.SEMICOLON);
                if (!this.test(Token.RPAREN)) {
                    node.Expression_c = this.parseExpression();
                }
                this.expect(Token.RPAREN);
                node.Statement = this.parseStatement();
                return this.finishNode(node, 'ForStatement');
            }
            const isLexicalStart = () => {
                switch (this.peekAhead().type) {
                    case Token.LBRACE:
                    case Token.LBRACK:
                    case Token.IDENTIFIER:
                    case Token.YIELD:
                    case Token.AWAIT:
                        return true;
                    default:
                        return false;
                }
            };
            if ((this.test('let') || this.test(Token.CONST)) && isLexicalStart()) {
                const inner = this.startNode();
                if (this.eat('let')) {
                    inner.LetOrConst = 'let';
                } else {
                    this.expect(Token.CONST);
                    inner.LetOrConst = 'const';
                }
                const list = this.parseBindingList();
                this.scope.declare(list, 'lexical');
                if (list.length > 1 || this.test(Token.SEMICOLON)) {
                    inner.BindingList = list;
                    node.LexicalDeclaration = this.finishNode(inner, 'LexicalDeclaration');
                    this.expect(Token.SEMICOLON);
                    if (!this.test(Token.SEMICOLON)) {
                        node.Expression_a = this.parseExpression();
                    }
                    this.expect(Token.SEMICOLON);
                    if (!this.test(Token.RPAREN)) {
                        node.Expression_b = this.parseExpression();
                    }
                    this.expect(Token.RPAREN);
                    node.Statement = this.parseStatement();
                    return this.finishNode(node, 'ForStatement');
                }
                inner.ForBinding = list[0];
                inner.ForBinding.type = 'ForBinding';
                if (inner.ForBinding.Initializer) {
                    this.unexpected(inner.ForBinding.Initializer);
                }
                node.ForDeclaration = this.finishNode(inner, 'ForDeclaration');
                getDeclarations(node.ForDeclaration)
                    .forEach((d) => {
                        if (d.name === 'let') {
                            this.raiseEarly('UnexpectedToken', d.node);
                        }
                    });
                if (!isAwait && this.eat(Token.IN)) {
                    node.Expression = this.parseExpression();
                    this.expect(Token.RPAREN);
                    node.Statement = this.parseStatement();
                    return this.finishNode(node, 'ForInStatement');
                }
                this.expect('of');
                node.AssignmentExpression = this.parseAssignmentExpression();
                this.expect(Token.RPAREN);
                node.Statement = this.parseStatement();
                return this.finishNode(node, isAwait ? 'ForAwaitStatement' : 'ForOfStatement');
            }
            if (this.eat(Token.VAR)) {
                if (isAwait) {
                    node.ForBinding = this.parseForBinding();
                    this.expect('of');
                    node.AssignmentExpression = this.parseAssignmentExpression();
                    this.expect(Token.RPAREN);
                    node.Statement = this.parseStatement();
                    return this.finishNode(node, 'ForAwaitStatement');
                }
                const list = this.parseVariableDeclarationList(false);
                if (list.length > 1 || this.test(Token.SEMICOLON)) {
                    node.VariableDeclarationList = list;
                    this.expect(Token.SEMICOLON);
                    if (!this.test(Token.SEMICOLON)) {
                        node.Expression_a = this.parseExpression();
                    }
                    this.expect(Token.SEMICOLON);
                    if (!this.test(Token.RPAREN)) {
                        node.Expression_b = this.parseExpression();
                    }
                    this.expect(Token.RPAREN);
                    node.Statement = this.parseStatement();
                    return this.finishNode(node, 'ForStatement');
                }
                node.ForBinding = list[0];
                node.ForBinding.type = 'ForBinding';
                if (node.ForBinding.Initializer) {
                    this.unexpected(node.ForBinding.Initializer);
                }
                if (this.eat('of')) {
                    node.AssignmentExpression = this.parseAssignmentExpression();
                } else {
                    this.expect(Token.IN);
                    node.Expression = this.parseExpression();
                }
                this.expect(Token.RPAREN);
                node.Statement = this.parseStatement();
                return this.finishNode(node, node.AssignmentExpression ? 'ForOfStatement' : 'ForInStatement');
            }

            this.scope.pushAssignmentInfo('for');
            const expression = this.scope.with({ in: false }, () => this.parseExpression());
            const validateLHS = (n) => {
                if (n.type === 'AssignmentExpression') {
                    this.raiseEarly('UnexpectedToken', n);
                } else {
                    this.validateAssignmentTarget(n);
                }
            };
            const assignmentInfo = this.scope.popAssignmentInfo();
            if (!isAwait && this.eat(Token.IN)) {
                assignmentInfo.clear();
                validateLHS(expression);
                node.LeftHandSideExpression = expression;
                node.Expression = this.parseExpression();
                this.expect(Token.RPAREN);
                node.Statement = this.parseStatement();
                return this.finishNode(node, 'ForInStatement');
            }
            if (this.eat('of')) {
                assignmentInfo.clear();
                validateLHS(expression);
                node.LeftHandSideExpression = expression;
                node.AssignmentExpression = this.parseAssignmentExpression();
                this.expect(Token.RPAREN);
                node.Statement = this.parseStatement();
                return this.finishNode(node, isAwait ? 'ForAwaitStatement' : 'ForOfStatement');
            }

            node.Expression_a = expression;
            this.expect(Token.SEMICOLON);

            if (!this.test(Token.SEMICOLON)) {
                node.Expression_b = this.parseExpression();
            }
            this.expect(Token.SEMICOLON);

            if (!this.test(Token.RPAREN)) {
                node.Expression_c = this.parseExpression();
            }
            this.expect(Token.RPAREN);

            node.Statement = this.parseStatement();
            return this.finishNode(node, 'ForStatement');
        });
    }

    // ForBinding :
    //   BindingIdentifier
    //   BindingPattern
    parseForBinding() {
        const node = this.startNode();
        switch (this.peek().type) {
            case Token.LBRACE:
            case Token.LBRACK:
                node.BindingPattern = this.parseBindingPattern();
                break;
            default:
                node.BindingIdentifier = this.parseBindingIdentifier();
                break;
        }
        return this.finishNode(node, 'ForBinding');
    }


    // SwitchStatement :
    //   `switch` `(` Expression `)` CaseBlock
    parseSwitchStatement() {
        const node = this.startNode();
        this.expect(Token.SWITCH);
        this.expect(Token.LPAREN);
        node.Expression = this.parseExpression();
        this.expect(Token.RPAREN);
        this.scope.with({
            lexical: true,
            label: 'switch',
        }, () => {
            node.CaseBlock = this.parseCaseBlock();
        });
        return this.finishNode(node, 'SwitchStatement');
    }

    // CaseBlock :
    //   `{` CaseClauses? `}`
    //   `{` CaseClauses? DefaultClause CaseClauses? `}`
    // CaseClauses :
    //   CaseClause
    //   CaseClauses CauseClause
    // CaseClause :
    //   `case` Expression `:` StatementList?
    // DefaultClause :
    //   `default` `:` StatementList?
    parseCaseBlock() {
        const node = this.startNode();
        this.expect(Token.LBRACE);
        while (!this.eat(Token.RBRACE)) {
            switch (this.peek().type) {
                case Token.CASE:
                case Token.DEFAULT: {
                    const inner = this.startNode();
                    const t = this.next().type;
                    if (t === Token.DEFAULT && node.DefaultClause) {
                        this.unexpected();
                    }
                    if (t === Token.CASE) {
                        inner.Expression = this.parseExpression();
                    }
                    this.expect(Token.COLON);
                    while (!(this.test(Token.CASE) || this.test(Token.DEFAULT) || this.test(Token.RBRACE))) {
                        if (!inner.StatementList) {
                            inner.StatementList = [];
                        }
                        inner.StatementList.push(this.parseStatementListItem());
                    }
                    if (t === Token.DEFAULT) {
                        node.DefaultClause = this.finishNode(inner, 'DefaultClause');
                    } else {
                        if (node.DefaultClause) {
                            if (!node.CaseClauses_b) {
                                node.CaseClauses_b = [];
                            }
                            node.CaseClauses_b.push(this.finishNode(inner, 'CaseClause'));
                        } else {
                            if (!node.CaseClauses_a) {
                                node.CaseClauses_a = [];
                            }
                            node.CaseClauses_a.push(this.finishNode(inner, 'CaseClause'));
                        }
                    }
                    break;
                }
                default:
                    this.unexpected();
            }
        }
        return this.finishNode(node, 'CaseBlock');
    }

    // BreakStatement :
    //   `break` `;`
    //   `break` [no LineTerminator here] LabelIdentifier `;`
    //
    // ContinueStatement :
    //   `continue` `;`
    //   `continue` [no LineTerminator here] LabelIdentifier `;`
    parseBreakContinueStatement() {
        const node = this.startNode();
        const isBreak = this.eat(Token.BREAK);
        if (!isBreak) {
            this.expect(Token.CONTINUE);
        }
        if (this.eat(Token.SEMICOLON)) {
            node.LabelIdentifier = null;
        } else if (this.peek().hadLineTerminatorBefore) {
            node.LabelIdentifier = null;
            this.semicolon();
        } else {
            if (this.test(Token.IDENTIFIER)) {
                node.LabelIdentifier = this.parseLabelIdentifier();
            } else {
                node.LabelIdentifier = null;
            }
            this.semicolon();
        }
        this.verifyBreakContinue(node, isBreak);
        return this.finishNode(node, isBreak ? 'BreakStatement' : 'ContinueStatement');
    }

    verifyBreakContinue(node, isBreak) {
        let i = 0;
        for (; i < this.scope.labels.length; i += 1) {
            const label = this.scope.labels[i];
            if (!node.LabelIdentifier || node.LabelIdentifier.name === label.name) {
                if (label.type && (isBreak || label.type === 'loop')) {
                    break;
                }
                if (node.LabelIdentifier && isBreak) {
                    break;
                }
            }
        }
        if (i === this.scope.labels.length) {
            this.raiseEarly('IllegalBreakContinue', node, isBreak);
        }
    }

    // ReturnStatement :
    //   `return` `;`
    //   `return` [no LineTerminator here] Expression `;`
    parseReturnStatement() {
        if (!this.scope.hasReturn()) {
            this.unexpected();
        }
        const node = this.startNode();
        this.expect(Token.RETURN);
        if (this.eat(Token.SEMICOLON)) {
            node.Expression = null;
        } else if (this.peek().hadLineTerminatorBefore) {
            node.Expression = null;
            this.semicolon();
        } else {
            node.Expression = this.parseExpression();
            this.semicolon();
        }
        return this.finishNode(node, 'ReturnStatement');
    }

    // WithStatement :
    //   `with` `(` Expression `)` Statement
    parseWithStatement() {
        if (this.isStrictMode()) {
            this.raiseEarly('UnexpectedToken');
        }
        const node = this.startNode();
        this.expect(Token.WITH);
        this.expect(Token.LPAREN);
        node.Expression = this.parseExpression();
        this.expect(Token.RPAREN);
        node.Statement = this.parseStatement();
        return this.finishNode(node, 'WithStatement');
    }

    // ThrowStatement :
    //   `throw` [no LineTerminator here] Expression `;`
    parseThrowStatement() {
        const node = this.startNode();
        this.expect(Token.THROW);
        if (this.peek().hadLineTerminatorBefore) {
            this.raise('NewlineAfterThrow', node);
        }
        node.Expression = this.parseExpression();
        this.semicolon();
        return this.finishNode(node, 'ThrowStatement');
    }

    // TryStatement :
    //   `try` Block Catch
    //   `try` Block Finally
    //   `try` Block Catch Finally
    //
    // Catch :
    //   `catch` `(` CatchParameter `)` Block
    //   `catch` Block
    //
    // Finally :
    //   `finally` Block
    //
    // CatchParameter :
    //   BindingIdentifier
    //   BindingPattern
    parseTryStatement() {
        const node = this.startNode();
        this.expect(Token.TRY);
        node.Block = this.parseBlock();
        if (this.eat(Token.CATCH)) {
            this.scope.with({ lexical: true }, () => {
                const clause = this.startNode();
                if (this.eat(Token.LPAREN)) {
                    switch (this.peek().type) {
                        case Token.LBRACE:
                        case Token.LBRACK:
                            clause.CatchParameter = this.parseBindingPattern();
                            break;
                        default:
                            clause.CatchParameter = this.parseBindingIdentifier();
                            break;
                    }
                    this.scope.declare(clause.CatchParameter, 'lexical');
                    this.expect(Token.RPAREN);
                } else {
                    clause.CatchParameter = null;
                }
                clause.Block = this.parseBlock(false);
                node.Catch = this.finishNode(clause, 'Catch');
            });
        } else {
            node.Catch = null;
        }
        if (this.eat(Token.FINALLY)) {
            node.Finally = this.parseBlock();
        } else {
            node.Finally = null;
        }
        if (!node.Catch && !node.Finally) {
            this.raise('TryMissingCatchOrFinally');
        }
        return this.finishNode(node, 'TryStatement');
    }

    // DebuggerStatement : `debugger` `;`
    parseDebuggerStatement() {
        const node = this.startNode();
        this.expect(Token.DEBUGGER);
        this.semicolon();
        return this.finishNode(node, 'DebuggerStatement');
    }

    // ExpressionStatement :
    //   [lookahead != `{`, `function`, `async` [no LineTerminator here] `function`, `class`, `let` `[` ] Expression `;`
    parseExpressionStatement() {
        switch (this.peek().type) {
            case Token.LBRACE:
            case Token.FUNCTION:
            case Token.CLASS:
                this.unexpected();
                break;
            default:
                if (this.test('async') && this.testAhead(Token.FUNCTION) && !this.peekAhead().hadLineTerminatorBefore) {
                    this.unexpected();
                }
                if (this.test('let') && this.testAhead(Token.LBRACK)) {
                    this.unexpected();
                }
                break;
        }
        const node = this.startNode();
        const expression = this.parseExpression();
        if (expression.type === 'IdentifierReference' && this.eat(Token.COLON)) {
            expression.type = 'LabelIdentifier';
            node.LabelIdentifier = expression;

            if (this.scope.labels.find((l) => l.name === node.LabelIdentifier.name)) {
                this.raiseEarly('AlreadyDeclared', node.LabelIdentifier, node.LabelIdentifier.name);
            }
            let type = null;
            switch (this.peek().type) {
                case Token.SWITCH:
                    type = 'switch';
                    break;
                case Token.DO:
                case Token.WHILE:
                case Token.FOR:
                    type = 'loop';
                    break;
                default:
                    break;
            }
            this.scope.labels.push({
                name: node.LabelIdentifier.name,
                type,
            });

            node.LabelledItem = this.parseStatement();

            this.scope.labels.pop();

            return this.finishNode(node, 'LabelledStatement');
        }
        node.Expression = expression;
        this.semicolon();
        return this.finishNode(node, 'ExpressionStatement');
    }

    // ImportDeclaration :
    //   `import` ImportClause FromClause `;`
    //   `import` ModuleSpecifier `;`
    parseImportDeclaration() {
        if (this.testAhead(Token.PERIOD) || this.testAhead(Token.LPAREN)) {
            // `import` `(`
            // `import` `.`
            return this.parseExpressionStatement();
        }
        const node = this.startNode();
        this.next();
        if (this.test(Token.STRING)) {
            node.ModuleSpecifier = this.parsePrimaryExpression();
        } else {
            node.ImportClause = this.parseImportClause();
            this.scope.declare(node.ImportClause, 'import');
            node.FromClause = this.parseFromClause();
        }
        this.semicolon();
        return this.finishNode(node, 'ImportDeclaration');
    }

    // ImportClause :
    //   ImportedDefaultBinding
    //   NameSpaceImport
    //   NamedImports
    //   ImportedDefaultBinding `,` NameSpaceImport
    //   ImportedDefaultBinding `,` NamedImports
    //
    // ImportedBinding :
    //   BindingIdentifier
    parseImportClause() {
        const node = this.startNode();
        if (this.test(Token.IDENTIFIER)) {
            node.ImportedDefaultBinding = this.parseImportedDefaultBinding();
            if (!this.eat(Token.COMMA)) {
                return this.finishNode(node, 'ImportClause');
            }
        }
        if (this.test(Token.MUL)) {
            node.NameSpaceImport = this.parseNameSpaceImport();
        } else if (this.eat(Token.LBRACE)) {
            node.NamedImports = this.parseNamedImports();
        } else {
            this.unexpected();
        }
        return this.finishNode(node, 'ImportClause');
    }

    // ImportedDefaultBinding :
    //   ImportedBinding
    parseImportedDefaultBinding() {
        const node = this.startNode();
        node.ImportedBinding = this.parseBindingIdentifier();
        return this.finishNode(node, 'ImportedDefaultBinding');
    }

    // NameSpaceImport :
    //   `*` `as` ImportedBinding
    parseNameSpaceImport() {
        const node = this.startNode();
        this.expect(Token.MUL);
        this.expect('as');
        node.ImportedBinding = this.parseBindingIdentifier();
        return this.finishNode(node, 'NameSpaceImport');
    }

    // NamedImports :
    //   `{` `}`
    //   `{` ImportsList `}`
    //   `{` ImportsList `,` `}`
    parseNamedImports() {
        const node = this.startNode();
        node.ImportsList = [];
        while (!this.eat(Token.RBRACE)) {
            node.ImportsList.push(this.parseImportSpecifier());
            if (this.eat(Token.RBRACE)) {
                break;
            }
            this.expect(Token.COMMA);
        }
        return this.finishNode(node, 'NamedImports');
    }

    // ImportSpecifier :
    //   ImportedBinding
    //   IdentifierName `as` ImportedBinding
    //   ModuleExportName `as` ImportedBinding
    parseImportSpecifier() {
        const node = this.startNode();
        if (this.feature('arbitrary-module-namespace-names') && this.test(Token.STRING)) {
            node.ModuleExportName = this.parseModuleExportName();
            this.expect('as');
            node.ImportedBinding = this.parseBindingIdentifier();
        } else {
            const name = this.parseIdentifierName();
            if (this.eat('as')) {
                node.IdentifierName = name;
                node.ImportedBinding = this.parseBindingIdentifier();
            } else {
                node.ImportedBinding = name;
                node.ImportedBinding.type = 'BindingIdentifier';
                if (isKeywordRaw(node.ImportedBinding.name)) {
                    this.raiseEarly('UnexpectedToken', node.ImportedBinding);
                }
                if (node.ImportedBinding.name === 'eval' || node.ImportedBinding.name === 'arguments') {
                    this.raiseEarly('UnexpectedToken', node.ImportedBinding);
                }
            }
        }
        return this.finishNode(node, 'ImportSpecifier');
    }

    // ExportDeclaration :
    //   `export` ExportFromClause FromClause `;`
    //   `export` NamedExports `;`
    //   `export` VariableStatement
    //   `export` Declaration
    //   `export` `default` HoistableDeclaration
    //   `export` `default` ClassDeclaration
    //   `export` `default` AssignmentExpression `;`
    //
    // ExportFromClause :
    //   `*`
    //   `*` as IdentifierName
    //   `*` as ModuleExportName
    //   NamedExports
    parseExportDeclaration() {
        const node = this.startNode();
        this.expect(Token.EXPORT);
        node.default = this.eat(Token.DEFAULT);
        if (node.default) {
            switch (this.peek().type) {
                case Token.FUNCTION:
                    node.HoistableDeclaration = this.scope.with({ default: true }, () => this.parseFunctionDeclaration(FunctionKind.NORMAL));
                    break;
                case Token.CLASS:
                    node.ClassDeclaration = this.scope.with({ default: true }, () => this.parseClassDeclaration());
                    break;
                default:
                    if (this.test('async') && this.testAhead(Token.FUNCTION) && !this.peekAhead().hadLineTerminatorBefore) {
                        node.HoistableDeclaration = this.scope.with({ default: true }, () => this.parseFunctionDeclaration(FunctionKind.ASYNC));
                    } else {
                        node.AssignmentExpression = this.parseAssignmentExpression();
                        this.semicolon();
                    }
                    break;
            }
            if (this.scope.exports.has('default')) {
                this.raiseEarly('AlreadyDeclared', node);
            } else {
                this.scope.exports.add('default');
            }
        } else {
            switch (this.peek().type) {
                case Token.CONST:
                    node.Declaration = this.parseLexicalDeclaration();
                    this.scope.declare(node.Declaration, 'export');
                    break;
                case Token.CLASS:
                    node.Declaration = this.parseClassDeclaration();
                    this.scope.declare(node.Declaration, 'export');
                    break;
                case Token.FUNCTION:
                    node.Declaration = this.parseHoistableDeclaration();
                    this.scope.declare(node.Declaration, 'export');
                    break;
                case Token.VAR:
                    node.VariableStatement = this.parseVariableStatement();
                    this.scope.declare(node.VariableStatement, 'export');
                    break;
                case Token.LBRACE: {
                    const NamedExports = this.parseNamedExports();
                    if (this.test('from')) {
                        node.ExportFromClause = NamedExports;
                        node.FromClause = this.parseFromClause();
                    } else {
                        NamedExports.ExportsList.forEach((n) => {
                            if (n.localName.type === 'StringLiteral') {
                                this.raiseEarly('UnexpectedToken', n.localName);
                            }
                        });
                        node.NamedExports = NamedExports;
                        this.scope.checkUndefinedExports(node.NamedExports);
                    }
                    this.semicolon();
                    break;
                }
                case Token.MUL: {
                    const inner = this.startNode();
                    this.next();
                    if (this.eat('as')) {
                        if (this.feature('arbitrary-module-namespace-names') && this.test(Token.STRING)) {
                            inner.ModuleExportName = this.parseModuleExportName();
                            this.scope.declare(inner.ModuleExportName, 'export');
                        } else {
                            inner.IdentifierName = this.parseIdentifierName();
                            this.scope.declare(inner.IdentifierName, 'export');
                        }
                    }
                    node.ExportFromClause = this.finishNode(inner, 'ExportFromClause');
                    node.FromClause = this.parseFromClause();
                    this.semicolon();
                    break;
                }
                default:
                    if (this.test('let')) {
                        node.Declaration = this.parseLexicalDeclaration();
                        this.scope.declare(node.Declaration, 'export');
                    } else if (this.test('async') && this.testAhead(Token.FUNCTION) && !this.peekAhead().hadLineTerminatorBefore) {
                        node.Declaration = this.parseHoistableDeclaration();
                        this.scope.declare(node.Declaration, 'export');
                    } else {
                        this.unexpected();
                    }
            }
        }
        return this.finishNode(node, 'ExportDeclaration');
    }

    // NamedExports :
    //   `{` `}`
    //   `{` ExportsList `}`
    //   `{` ExportsList `,` `}`
    parseNamedExports() {
        const node = this.startNode();
        this.expect(Token.LBRACE);
        node.ExportsList = [];
        while (!this.eat(Token.RBRACE)) {
            node.ExportsList.push(this.parseExportSpecifier());
            if (this.eat(Token.RBRACE)) {
                break;
            }
            this.expect(Token.COMMA);
        }
        return this.finishNode(node, 'NamedExports');
    }

    // ExportSpecifier :
    //   IdentifierName
    //   IdentifierName `as` IdentifierName
    //   IdentifierName `as` ModuleExportName
    //   ModuleExportName
    //   ModuleExportName `as` ModuleExportName
    //   ModuleExportName `as` IdentifierName
    parseExportSpecifier() {
        const node = this.startNode();
        const parseName = () => {
            if (this.feature('arbitrary-module-namespace-names') && this.test(Token.STRING)) {
                return this.parseModuleExportName();
            }
            return this.parseIdentifierName();
        };
        node.localName = parseName();
        if (this.eat('as')) {
            node.exportName = parseName();
        } else {
            node.exportName = node.localName;
        }
        this.scope.declare(node.exportName, 'export');
        return this.finishNode(node, 'ExportSpecifier');
    }

    // ModuleExportName : StringLiteral
    parseModuleExportName() {
        const literal = this.parseStringLiteral();
        if (!IsStringValidUnicode(StringValue(literal))) {
            this.raiseEarly('ModuleExportNameInvalidUnicode', literal);
        }
        return literal;
    }

    // FromClause :
    //   `from` ModuleSpecifier
    parseFromClause() {
        this.expect('from');
        return this.parseStringLiteral();
    }
}
