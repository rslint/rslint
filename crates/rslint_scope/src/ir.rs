use std::cell::RefCell;
use std::rc::{Rc, Weak};

use rslint_parser::{ast::Expr, *};

/// A variable which exists in a particular scope.
#[derive(Debug, Clone)]
pub(crate) struct VariableBinding {
    pub(crate) declarations: Vec<VariableDeclaration>,
    pub(crate) name: String,
    /// All the references to this variable
    pub(crate) references: Vec<Weak<RefCell<VariableRef>>>,
    pub(crate) id: usize,
    // can't be an Arc or we would have a reference cycle which leaks memory
    pub(crate) scope: Weak<RefCell<Scope>>,
}

impl VariableBinding {
    pub fn is_function_scoped(&self) -> bool {
        self.declarations
            .first()
            .map_or(false, |x| matches!(x.kind, BindingKind::Var(_)))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct VariableDeclaration {
    pub(crate) node: SyntaxNode,
    pub(crate) initial_value: Option<Expr>,
    pub(crate) kind: BindingKind,
}

#[derive(Debug, Clone)]
pub(crate) struct VariableRef {
    /// The node which houses the variable reference, e.g. `foo` in `foo + bar`.
    pub(crate) node: SyntaxNode,
    /// How the variable was used.
    pub(crate) usage: VariableUsageKind,
    /// The variable declaration, if the variable was actually defined.
    pub(crate) declaration: Option<Weak<RefCell<VariableBinding>>>,
}

#[derive(Debug, Clone)]
pub(crate) struct Scope {
    pub(crate) node: SyntaxNode,
    pub(crate) kind: ScopeKind,
    pub(crate) variables: Vec<Rc<RefCell<VariableBinding>>>,
    pub(crate) var_refs: Vec<Rc<RefCell<VariableRef>>>,
    pub(crate) children: Vec<Rc<RefCell<Scope>>>,
    pub(crate) parent: Option<Weak<RefCell<Scope>>>,
    pub(crate) strict: bool,
}

impl Scope {
    pub fn is_child_of(&self, other: &Scope) -> bool {
        if std::ptr::eq(self, other) {
            return false;
        }
        other
            .node
            .text_range()
            .contains_range(self.node.text_range())
    }
}

/// How a variable was used in a reference to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum VariableUsageKind {
    /// Usages such as `foo.bar`, `let a = foo`, etc.
    Read,
    /// Calling the variable as a function, like `foo()`.
    Call,
    /// Constructing a new instance of the variable, like `new foo()`.
    Construct,
    /// Reassign the variable
    Write,
}

/// In what way a variable is declared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BindingKind {
    /// An import declaration, includes the type of import and the optional source.
    Import(ImportBindingKind, Option<SmolStr>),
    Const(PatternBindingKind),
    Let(PatternBindingKind),
    Var(PatternBindingKind),
    Class,
    Function,
    Param(PatternBindingKind),
    CatchClause,
    Getter,
    Setter,
    Arguments,
    Method,
}

/// The kind of pattern which declares a parameter or variable or destructuring assignment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PatternBindingKind {
    /// A single identifier binding such as `let foo = 5`
    Literal,
    /// An object destructured binding such as `let { foo, bar } = foo`
    Object,
    /// An array destructured binding such as `let [ foo, bar ] = foo`
    Array,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ImportBindingKind {
    /// An import such as `import * as foo from "a"`
    NamedWildcard,
    /// An import from named imports, such as `import { foo, bar } from "a"`
    /// also includes the optional original name if this is an alias, aka `foo as bar`
    DestructuredImport(Option<SmolStr>),
    /// An import referencing a default, such as `import foo from "bar"`
    LiteralImport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ScopeKind {
    Arrow,
    Function,
    Block,
    Loop,
    Class,
    Switch,
    With,
    Catch,
    Global,
    Getter,
    Setter,
    Method,
}
