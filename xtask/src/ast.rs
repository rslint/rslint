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
        ("&", "AMP"),
        ("|", "PIPE"),
        ("+", "PLUS"),
        ("++", "PLUS2"),
        ("*", "STAR"),
        ("/", "SLASH"),
        ("^", "CARET"),
        ("%", "PERCENT"),
        (".", "DOT"),
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
    ],
    keywords: &[
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
        "let",
        "new",
        "null",
        "package",
        "private",
        "protected",
        "public",
        "return",
        "static",
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
        "yield"
    ],
    literals: &[
        "NUMBER",
        "STRING",
        "REGEX",
    ],
    tokens: &[
        "ERROR",
        "IDENT",
        "WHITESPACE",
        "COMMENT",
        "SHEBANG"
    ],
    nodes: &[
        "PROGRAM",
        "BLOCK_STMT",
        "VAR_STMT",
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
        "LABELLED_STMT",
        "THROW_STMT",
        "TRY_STMT",
        "CATCH_CLAUSE",
        "DEBUGGER_STMT",
        "FN_DECL",
        "NAME",
        "FN_BODY",
        "PARAMETER_LIST",
        "THIS_EXPR",
        "ARRAY_EXPR",
        "OBJECT_EXPR",
        "LITERAL_PROP",
        "GETTER_PROP",
        "SETTER_PROP",
        "GROUPING_EXPR",
        "NEW_EXPR",
        "FN_EXPR",
        "BRACKET_EXPR",
        "DOT_EXPR",
        "CALL_EXPR",
        "POSTFIX_EXPR",
        "UNARY_EXPR",
        "BIN_EXPR",
        "COND_EXPR",
        "ASSIGN_EXPR",
        "SEQUENCE_EXPR",
        "ARG_LIST",
        "LITERAL",
        "CONDITION",
    ]
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
            $($field_name:ident $(![$token:tt])? $(: $ty:tt)?),*$(,)?
        }
    )*) => {
        [$(
            AstNodeSrc {
                name: stringify!($name),
                fields: &[
                    $(field!($(T![$token])? $field_name $($ty)?)),*
                ],
            }
        ),*]
    };
}

macro_rules! field {
    (T![$token:tt] T) => {
        Field::Token(stringify!($token))
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
        struct Program {
            items: [StmtListItem],
        }

        struct Literal { /*LiteralToken*/ }

        struct BlockStmt {
            T!['{'],
            stmts: [Stmt],
            T!['}'],
        }

        struct VarStmt {
            T![var],
            declared: [Declarator],
            T![;],
        }

        struct Declarator {
            T![ident],
            T![=],
            value: AssignExpr,
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
            cons: Stmt,
            T![else],
            alt: Stmt,
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
            init: ForHead,
            /* semicolon */
            test: Expr,
            /* semicolon */
            update: Expr,
            T![')'],
            cons: Stmt,
        }

        struct ForInStmt {
            T![for],
            T!['('],
            left: ForHead,
            T![in],
            right: Expr,
            T![')'],
            cons: Stmt,
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
            cases: [CaseClause],
            T!['}'],
        }

        struct CaseClause {
            T![default],
            T![case],
            test: Expr,
            T![:],
            cons: [Stmt],
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
            T![finally],
            finalizer: BlockStmt,
        }

        struct CatchClause {
            T![catch],
            T!['('],
            error: Name,
            T![')'],
            cons: BlockStmt
        }

        struct DebuggerStmt {
            T![debugger],
            T![;],
        }

        struct FnDecl {
            T![function],
            name: Name,
            parameters: ParameterList,
            body: FnBody,
        }

        struct Name { T![ident] }

        struct ParameterList {
            T!['('],
            parameters: [Name],
            T![')'],
        }

        struct FnBody {
            T!['{'],
            body: [StmtListItem],
            T!['}'],
        }

        struct ThisExpr {
            T![this],
        }

        struct ArrayExpr {
            T!['['],
            elements: [Expr],
            T![']'],
        }

        struct ObjectExpr {
            T!['{'],
            props: [ObjectProp],
            T!['}'],
        }

        struct LiteralProp {
            /* key */
            T![:]
            /* value */
        }

        struct GetterProp {
            T![ident],
            key: Literal,
            parameters: ParameterList,
            body: FnBody,
        }

        struct SetterProp {
            key: Literal,
            parameters: ParameterList,
            body: FnBody,
        }

        struct GroupingExpr {
            T!['('],
            inner: Expr,
            T![')'],
        }

        struct FnExpr {
            T![function],
            name: Name,
            parameters: ParameterList,
            body: FnBody,
        }

        struct BracketExpr {
            /* object */
            T!['['],
            /* prop */
            T![']'],
        }

        struct DotExpr {
            object: Expr,
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
            args: [Expr],
            T![')'],
        }

        struct CallExpr {
            callee: Expr,
            arguments: ArgList,
        }

        struct PostfixExpr {
            Expr,
            /* Postfix op */
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

        // Perhaps we should merge this into binexpr?
        struct AssignExpr {
            /* AssignOp */
        }

        struct SequenceExpr {
            exprs: [Expr],
        }
    },
    enums: &ast_enums!{
        enum ObjectProp {
            LiteralProp,
            GetterProp,
            SetterProp,
        }

        /* 
        enum StmtListItem {
            STMT,
            DECLARATION
        }
        */
        
        enum Declaration {
            FnDecl
        }

        enum Stmt {
            BlockStmt,
            VarStmt,
            EmptyStmt,
            ExprStmt,
            IfStmt,
            DoWhileStmt,
            WhileStmt,
            ForStmt,
            ForInStmt,
            ContinueStmt,
            BreakStmt,
            ReturnStmt,
            WithStmt,
            LabelledStmt,
            SwitchStmt,
            ThrowStmt,
            TryStmt,
            DebuggerStmt
        }

        /* 
        enum ForHead {
            VAR_STMT,
            EXPR
        }
        */

        enum Expr {
            Literal,
            Name,
            ThisExpr,
            ArrayExpr,
            ObjectExpr,
            GroupingExpr,
            BracketExpr,
            DotExpr,
            NewExpr,
            CallExpr,
            PostfixExpr,
            UnaryExpr,
            BinExpr,
            CondExpr,
            AssignExpr,
            SequenceExpr,
        }
    }
};