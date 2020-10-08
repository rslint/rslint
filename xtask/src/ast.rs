//! Definitions for the ECMAScript AST used for codegen
//! Based on the rust analyzer parser and ast definitions

pub(crate) struct KindsSrc<'a> {
    pub(crate) punct: &'a [(&'a str, &'a str)],
    pub(crate) keywords: &'a [&'a str],
    pub(crate) literals: &'a [&'a str],
    pub(crate) tokens: &'a [&'a str],
    pub(crate) nodes: &'a [&'a str],
}

pub(crate) const KINDS_SRC: KindsSrc = KindsSrc {
    punct: &[
        (";", "SEMICOLON"),
        (",", "COMMA"),
        ("(", "L_PAREN"),
        (")", "R_PAREN"),
        ("{", "L_CURLY"),
        ("}", "R_CURLY"),
        ("[", "L_BRACK"),
        ("]", "R_BRACK"),
        ("<", "L_ANGLE"),
        (">", "R_ANGLE"),
        ("~", "TILDE"),
        ("?", "QUESTION"),
        ("??", "QUESTION2"),
        // These are *not* question AND dot tokens, they are one
        // to distinguish between `? .3134` and `?.` per ecma specs
        ("?.", "QUESTIONDOT"),
        ("&", "AMP"),
        ("|", "PIPE"),
        ("+", "PLUS"),
        ("++", "PLUS2"),
        ("*", "STAR"),
        ("**", "STAR2"),
        ("/", "SLASH"),
        ("^", "CARET"),
        ("%", "PERCENT"),
        (".", "DOT"),
        ("...", "DOT2"),
        (":", "COLON"),
        ("=", "EQ"),
        ("==", "EQ2"),
        ("===", "EQ3"),
        ("=>", "FAT_ARROW"),
        ("!", "BANG"),
        ("!=", "NEQ"),
        ("!==", "NEQ2"),
        ("-", "MINUS"),
        ("--", "MINUS2"),
        ("<=", "LTEQ"),
        (">=", "GTEQ"),
        ("+=", "PLUSEQ"),
        ("-=", "MINUSEQ"),
        ("|=", "PIPEEQ"),
        ("&=", "AMPEQ"),
        ("^=", "CARETEQ"),
        ("/=", "SLASHEQ"),
        ("*=", "STAREQ"),
        ("%=", "PERCENTEQ"),
        ("&&", "AMP2"),
        ("||", "PIPE2"),
        ("<<", "SHL"),
        (">>", "SHR"),
        (">>>", "USHR"),
        ("<<=", "SHLEQ"),
        (">>=", "SHREQ"),
        (">>>=", "USHREQ"),
        ("&&=", "AMP2EQ"),
        ("||=", "PIPE2EQ"),
        ("**=", "STAR2EQ"),
        ("??=", "QUESTION2EQ"),
    ],
    keywords: &[
        "await",
        "break",
        "case",
        "catch",
        "class",
        "const",
        "continue",
        "debugger",
        "default",
        "delete",
        "do",
        "else",
        "enum",
        "export",
        "extends",
        "false",
        "finally",
        "for",
        "function",
        "if",
        "in",
        "instanceof",
        "interface",
        "import",
        "implements",
        "new",
        "null",
        "package",
        "private",
        "protected",
        "public",
        "return",
        "super",
        "switch",
        "this",
        "throw",
        "try",
        "true",
        "typeof",
        "var",
        "void",
        "while",
        "with",
        "yield",
    ],
    literals: &["NUMBER", "STRING", "REGEX"],
    tokens: &[
        "TEMPLATE_CHUNK",
        "DOLLARCURLY", // ${
        "BACKTICK",
        "ERROR_TOKEN",
        "IDENT",
        "WHITESPACE",
        "COMMENT",
        "SHEBANG",
    ],
    nodes: &[
        "SCRIPT",
        "MODULE",
        "ERROR",
        "BLOCK_STMT",
        "VAR_DECL",
        "DECLARATOR",
        "EMPTY_STMT",
        "EXPR_STMT",
        "IF_STMT",
        "DO_WHILE_STMT",
        "WHILE_STMT",
        "FOR_STMT",
        "FOR_IN_STMT",
        "CONTINUE_STMT",
        "BREAK_STMT",
        "RETURN_STMT",
        "WITH_STMT",
        "SWITCH_STMT",
        "CASE_CLAUSE",
        "DEFAULT_CLAUSE",
        "LABELLED_STMT",
        "THROW_STMT",
        "TRY_STMT",
        "CATCH_CLAUSE",
        "FINALIZER",
        "DEBUGGER_STMT",
        "FN_DECL",
        "NAME",
        "NAME_REF",
        "PARAMETER_LIST",
        "THIS_EXPR",
        "ARRAY_EXPR",
        "OBJECT_EXPR",
        "LITERAL_PROP",
        "GETTER",
        "SETTER",
        "GROUPING_EXPR",
        "NEW_EXPR",
        "FN_EXPR",
        "BRACKET_EXPR",
        "DOT_EXPR",
        "CALL_EXPR",
        "UNARY_EXPR",
        "BIN_EXPR",
        "COND_EXPR",
        "ASSIGN_EXPR",
        "SEQUENCE_EXPR",
        "ARG_LIST",
        "LITERAL",
        "TEMPLATE",
        "TEMPLATE_ELEMENT",
        "CONDITION",
        "SPREAD_ELEMENT",
        "SUPER_CALL",
        "IMPORT_CALL",
        "NEW_TARGET",
        "IMPORT_META",
        "IDENT_PROP",
        "SPREAD_PROP",
        "INITIALIZED_PROP",
        "OBJECT_PATTERN",
        "ARRAY_PATTERN",
        "ASSIGN_PATTERN",
        "REST_PATTERN",
        "KEY_VALUE_PATTERN",
        "COMPUTED_PROPERTY_NAME",
        "FOR_OF_STMT",
        "SINGLE_PATTERN",
        "ARROW_EXPR",
        "YIELD_EXPR",
        "STATIC_METHOD",
        "CLASS_DECL",
        "CLASS_EXPR",
        "CLASS_BODY",
        "METHOD",
        "IMPORT_DECL",
        "EXPORT_DECL",
        "EXPORT_NAMED",
        "EXPORT_DEFAULT_DECL",
        "EXPORT_DEFAULT_EXPR",
        "EXPORT_WILDCARD",
        "WILDCARD_IMPORT",
        "NAMED_IMPORTS",
        "SPECIFIER",
        "AWAIT_EXPR",
        // These three are just hacks for converting to ast node without
        // having to handle every error recovery case.
        // in the future we might just tag the underlying rowan nodes
        "FOR_STMT_TEST",
        "FOR_STMT_UPDATE",
        "FOR_STMT_INIT",
        // Regex, rslint_syntax includes those defs too so we dont have a weird
        // interface between rslint_regex syntax kinds and rslint_syntax syntax kinds
    ],
};

pub(crate) struct AstSrc<'a> {
    pub(crate) tokens: &'a [&'a str],
    pub(crate) nodes: &'a [AstNodeSrc<'a>],
    pub(crate) enums: &'a [AstEnumSrc<'a>],
}

pub(crate) struct AstNodeSrc<'a> {
    pub(crate) name: &'a str,
    pub(crate) fields: &'a [Field<'a>],
}

pub(crate) enum Field<'a> {
    Token(&'a str),
    Node { name: &'a str, src: FieldSrc<'a> },
}

pub(crate) enum FieldSrc<'a> {
    Shorthand,
    Optional(&'a str),
    Many(&'a str),
}

pub(crate) struct AstEnumSrc<'a> {
    pub(crate) name: &'a str,
    pub(crate) variants: &'a [&'a str],
}

macro_rules! ast_nodes {
    ($(
        struct $name:ident {
            $($field_name:ident $(![$($token:tt)*])? $(: $ty:tt)?),*$(,)?
        }
    )*) => {
        [$(
            AstNodeSrc {
                name: stringify!($name),
                fields: &[
                    $(field!($(T![$($token)*])? $field_name $($ty)?)),*
                ],
            }
        ),*]
    };
}

macro_rules! field {
    (T![$($token:tt)*] T) => {
        Field::Token(stringify!($($token)*))
    };
    ($field_name:ident) => {
        Field::Node { name: stringify!($field_name), src: FieldSrc::Shorthand }
    };
    ($field_name:ident [$ty:ident]) => {
        Field::Node { name: stringify!($field_name), src: FieldSrc::Many(stringify!($ty)) }
    };
    ($field_name:ident $ty:ident) => {
        Field::Node { name: stringify!($field_name), src: FieldSrc::Optional(stringify!($ty)) }
    };
}

macro_rules! ast_enums {
    ($(
        enum $name:ident {
            $($variant:ident),*$(,)?
        }
    )*) => {
        [$(
            AstEnumSrc {
                name: stringify!($name),
                variants: &[$(stringify!($variant)),*],
            }
        ),*]
    };
}

/// Data used by codegen for generating ast nodes and SyntaxKind enums.  
/// Comments represent definitions which are manually created since they are either unique enough
/// or special enough to generate definitions for manually.
pub(crate) const AST_SRC: AstSrc = AstSrc {
    tokens: &["Whitespace", "Comment", "String"],
    nodes: &ast_nodes! {
        struct Script {
            items: [Stmt],
        }

        struct Module {
            items: [ModuleItem],
        }

        struct ImportDecl {
            T![import],
            imports: [ImportClause],
            /* from */
            /* source */
            T![;]
        }

        struct WildcardImport {
            T![*],
            /* as */
            /* alias */
        }

        struct NamedImports {
            T!['{'],
            specifiers: [Specifier],
            T!['}']
        }

        struct Specifier {
            /* manual impl */
        }

        struct ExportDecl {
            T![export],
            decl: Decl
        }

        struct ExportNamed {
            T![export],
            T!['{'],
            specifiers: [Specifier],
            T!['}'],
            /* from */
            /* source */
        }

        struct ExportWildcard {
            T![export],
            T![*],
            /* as */
            /* name */
            /* from */
            /* source */
        }

        struct ExportDefaultDecl {
            T![export],
            T![default],
            decl: DefaultDecl
        }

        struct ExportDefaultExpr {
            T![export],
            T![default],
            expr: Expr
        }

        struct Literal { /*LiteralToken*/ }

        struct BlockStmt {
            T!['{'],
            stmts: [Stmt],
            T!['}'],
        }

        struct VarDecl {
            T![var],
            /* let */
            T![const],
            declared: [Declarator],
            T![;],
        }

        struct Declarator {
            pattern: Pattern,
            T![=],
            value: Expr,
        }

        struct EmptyStmt {
            T![;],
        }

        struct ExprStmt {
            expr: Expr,
        }

        struct IfStmt {
            T![if],
            condition: Condition,
            /* cons */
            T![else],
            /* alt */
        }

        struct Condition {
            T!['('],
            condition: Expr,
            T![')'],
        }

        struct DoWhileStmt {
            T![do],
            cons: Stmt,
            T![while],
            condition: Condition,
            T![;],
        }

        struct WhileStmt {
            T![while],
            condition: Condition,
            cons: Stmt,
        }

        struct ForStmt {
            T![for],
            T!['('],
            init: ForStmtInit,
            /* semicolon */
            test: ForStmtTest,
            /* semicolon */
            update: ForStmtUpdate,
            T![')'],
            cons: Stmt,
        }

        struct ForStmtInit {
            inner: ForHead
        }

        struct ForStmtTest {
            expr: Expr
        }

        struct ForStmtUpdate {
            expr: Expr
        }

        struct ForInStmt {
            T![for],
            T!['('],
            left: ForStmtInit,
            T![in],
            right: Expr,
            T![')'],
            cons: Stmt,
        }

        struct ForOfStmt {
            T![for],
            T![await],
            T!['('],
            left: ForStmtInit,
            /* of */
            right: Expr,
            T![')'],
            cons: Stmt
        }

        struct ContinueStmt {
            T![continue],
            T![ident],
            T![;],
        }

        struct BreakStmt {
            T![break],
            T![ident],
            T![;],
        }

        struct ReturnStmt {
            T![return],
            value: Expr,
            T![;],
        }

        struct WithStmt {
            T![with],
            condition: Condition,
            cons: Stmt,
        }

        struct SwitchStmt {
            T![switch],
            test: Condition,
            T!['{'],
            cases: [SwitchCase],
            T!['}'],
        }

        struct CaseClause {
            T![case],
            test: Expr,
            T![:],
            cons: [Stmt],
        }

        struct DefaultClause {
            T![default],
            T![:],
            cons: [Stmt]
        }

        struct LabelledStmt {
            label: Name,
            T![:],
            stmt: Stmt,
        }

        struct ThrowStmt {
            T![throw],
            exception: Expr,
            T![;],
        }

        struct TryStmt {
            T![try],
            test: BlockStmt,
            handler: CatchClause,
            finalizer: Finalizer,
        }

        struct CatchClause {
            T![catch],
            T!['('],
            error: Pattern,
            T![')'],
            cons: BlockStmt
        }

        struct Finalizer {
            T![finally],
            cons: BlockStmt
        }

        struct DebuggerStmt {
            T![debugger],
            T![;],
        }

        struct FnDecl {
            /* async */
            T![function],
            T![*],
            name: Name,
            parameters: ParameterList,
            body: BlockStmt,
        }

        struct Name { T![ident] }

        struct NameRef { T![ident] }

        struct ParameterList {
            T!['('],
            parameters: [Pattern],
            T![')'],
        }

        struct ThisExpr {
            T![this],
        }

        struct ArrayExpr {
            T!['['],
            elements: [ExprOrSpread],
            T![']'],
        }

        struct ObjectExpr {
            T!['{'],
            props: [ObjectProp],
            T!['}'],
        }

        struct SpreadProp {
            T![...],
            value: Expr,
        }

        struct InitializedProp {
            key: Name,
            T![=],
            value: Expr,
        }

        struct IdentProp {
            name: Name
        }

        struct LiteralProp {
            /* key */
            T![:]
            /* value */
        }

        struct Getter {
            /* get */
            key: PropName,
            T!['('],
            T![')'],
            body: BlockStmt,
        }

        struct Setter {
            /* set */
            key: PropName,
            parameters: ParameterList,
            body: BlockStmt,
        }

        struct GroupingExpr {
            T!['('],
            inner: Expr,
            T![')'],
        }

        struct BracketExpr {
            T![super],
            /* object */
            /* optional chain */
            T!['['],
            /* prop */
            T![']'],
        }

        struct DotExpr {
            T![super],
            object: Expr,
            /* optional chain */
            T![.],
            prop: Name,
        }

        struct NewExpr {
            T![new],
            object: Expr,
            arguments: ArgList,
        }

        struct ArgList {
            T!['('],
            // TODO: Change this to expr or spread
            args: [Expr],
            T![')'],
        }

        struct CallExpr {
            callee: Expr,
            /* optional chain */
            arguments: ArgList,
        }

        struct SuperCall {
            T![super],
            arguments: ArgList
        }

        struct ImportCall {
            T![import],
            // This only takes one arg, it doesnt take an ArgList
            T!['('],
            argument: Expr,
            T![')']
        }

        struct NewTarget {
            T![new],
            T![.],
            /* target */
        }

        struct ImportMeta {
            T![import],
            T![.],
            /* meta */
        }

        struct UnaryExpr {
            /* Prefix op */
            Expr,
        }

        struct BinExpr {
            /* Binop */
        }

        struct CondExpr {
            /* test */
            T![?],
            /* cons */
            T![:],
            /* alt */
        }

        struct AssignExpr {
            /* lhs: PatternOrExpr, */
            /* AssignOp */
            /* rhs: Expr */
        }

        struct SequenceExpr {
            exprs: [Expr],
        }

        struct Template {
            tag: Expr,
            /* backtick */
            /* chunks */
            elements: [TemplateElement],
            /* backtick */
        }

        struct TemplateElement {
            /* dollarcurly */
            expr: Expr,
            T!['}']
        }

        struct SpreadElement {
            T![...],
            element: Expr
        }

        struct ArrayPattern {
            T!['['],
            elements: [Pattern],
            T![']']
        }

        struct ObjectPattern {
            T!['{'],
            elements: [ObjectPatternProp],
            T!['}']
        }

        struct RestPattern {
            T![...],
            pat: Pattern
        }

        struct AssignPattern {
            key: Pattern,
            T![=],
            value: Expr
        }

        struct KeyValuePattern {
            key: PropName,
            T![:],
            /* pattern */
        }

        struct ComputedPropertyName {
            T!['['],
            prop: Expr,
            T![']']
        }

        struct SinglePattern {
            name: Name
        }

        struct ArrowExpr {
            /* async */
            params: ArrowExprParams,
            T![=>],
            /* ExprOrBlock */
        }

        struct YieldExpr {
            T![yield],
            T![*],
            value: Expr
        }

        struct FnExpr {
            /* async */
            T![function],
            T![*],
            name: Name,
            parameters: ParameterList,
            body: BlockStmt,
        }

        struct Method {
            /* async */
            T![*],
            name: PropName,
            parameters: ParameterList,
            body: BlockStmt
        }

        struct StaticMethod {
            T![ident],
            method: Method
        }

        struct ClassDecl {
            T![class],
            name: Name,
            T![extends],
            parent: Expr,
            body: ClassBody
        }

        struct ClassExpr {
            T![class],
            /* name */
            T![extends],
            /* parent */
            body: ClassBody
        }

        struct ClassBody {
            T!['{'],
            elements: ClassElement,
            T!['}']
        }

        struct AwaitExpr {
            T![await],
            expr: Expr
        }
    },
    enums: &ast_enums! {
        enum ObjectProp {
            LiteralProp,
            Getter,
            Setter,
            SpreadProp,
            InitializedProp,
            IdentProp,
            Method
        }

        enum Pattern {
            SinglePattern,
            RestPattern,
            AssignPattern,
            ObjectPattern,
            ArrayPattern
        }

        enum SwitchCase {
            CaseClause,
            DefaultClause,
        }

        enum ObjectPatternProp {
            AssignPattern,
            KeyValuePattern,
            RestPattern,
            SinglePattern,
        }

        enum ArrowExprParams {
            Name,
            ParameterList
        }

        enum MethodDefinition {
            Method,
            Getter,
            Setter
        }

        enum ClassElement {
            EmptyStmt,
            Method,
            StaticMethod
        }

        enum ImportClause {
            WildcardImport,
            NamedImports,
            Name
        }

        enum DefaultDecl {
            FnDecl,
            ClassDecl
        }

        /*
        enum ModuleItem {
            Stmt,
            ImportDeclaration,
            ExportDeclaration
        }
        */

        /*
        enum ExprOrSpread {
            Expr,
            SpreadElement
        }
        */

        /*
        enum StmtListItem {
            STMT,
            DECLARATION
        }
        */

        enum Decl {
            FnDecl,
            ClassDecl,
            VarDecl,
        }

        /*
        enum ForHead {
            VAR_STMT,
            EXPR
        }
        */

        enum Expr {
            ArrowExpr,
            Literal,
            Template,
            NameRef,
            ThisExpr,
            ArrayExpr,
            ObjectExpr,
            GroupingExpr,
            BracketExpr,
            DotExpr,
            NewExpr,
            CallExpr,
            UnaryExpr,
            BinExpr,
            CondExpr,
            AssignExpr,
            SequenceExpr,
            FnExpr,
            ClassExpr,
            NewTarget,
            ImportMeta,
            SuperCall,
            ImportCall,
            YieldExpr,
            AwaitExpr
        }
    },
};
