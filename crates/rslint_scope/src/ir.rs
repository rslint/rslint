use std::ops::Deref;
use std::sync::{Arc, Weak};
use std::{borrow::Borrow, cell::RefCell};

use rslint_parser::{ast::Expr, *};

// SAFETY: The refcell usage for scopes is only required when actually running the analyzer.
// after the analyzer is finished, we guarantee 100% that zero mutable references can exist to any of the refcells
// because the fields are pub(crate), therefore it is ok for us to just borrow without checking because of
// this guarantee.
unsafe fn borrow_unchecked<T>(refcell: &RefCell<T>) -> &T {
    &*refcell.as_ptr()
}

/// A variable which exists in a particular scope.
#[derive(Debug, Clone)]
pub struct VariableBinding {
    pub declarations: Vec<VariableDeclaration>,
    pub name: String,
    /// All the references to this variable
    pub(crate) references: Vec<Arc<RefCell<VariableRef>>>,
    pub id: usize,
    // can't be an Arc or we would have a reference cycle which leaks memory
    pub(crate) scope: Weak<RefCell<Scope>>,
}

impl VariableBinding {
    pub fn is_function_scoped(&self) -> bool {
        self.declarations
            .first()
            .map_or(false, |x| matches!(x.kind, BindingKind::Var(_)))
    }

    /// Returns the scope this variable binding was declared in.
    ///
    /// # Panics
    ///
    /// Panics if the parent scope was dropped.
    pub fn scope(&self) -> Scope {
        unsafe {
            // we cant return `&Scope` because we would be returning a temporary value.
            // nor can we return `Rc<RefCell<Scope>>` since that would be unsound because
            // of the invariants we established in `borrow_unchecked`
            borrow_unchecked(&Weak::upgrade(&self.scope).expect("scope dropped prematurely"))
                .clone()
        }
    }

    /// Returns all of the references to this variable definition.
    pub fn references(&self) -> impl Iterator<Item = &VariableRef> {
        self.references
            .iter()
            .map(|x| unsafe { borrow_unchecked(x.deref()) })
    }
}

#[derive(Debug, Clone)]
pub struct VariableDeclaration {
    pub node: SyntaxNode,
    pub initial_value: Option<Expr>,
    pub kind: BindingKind,
}

#[derive(Debug, Clone)]
pub struct VariableRef {
    /// The node which houses the variable reference, e.g. `foo` in `foo + bar`.
    pub node: SyntaxNode,
    /// How the variable was used.
    pub usage: VariableUsageKind,
    /// The variable declaration, if the variable was actually defined.
    pub(crate) declaration: Option<Weak<RefCell<VariableBinding>>>,
    /// The name of the variable referenced
    pub name: String,
}

impl VariableRef {
    /// Returns the declaration of the variable, if it actually exists.
    pub fn declaration(&self) -> Option<VariableBinding> {
        unsafe {
            self.declaration.as_ref().map(|x| {
                borrow_unchecked(&Weak::upgrade(&x).expect("scope dropped prematurely")).clone()
            })
        }
    }

    /// Whether this variable reference references an undefined variable
    pub fn is_undef(&self) -> bool {
        self.declaration.is_none()
    }
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub node: SyntaxNode,
    pub kind: ScopeKind,
    pub(crate) variables: Vec<Arc<RefCell<VariableBinding>>>,
    pub(crate) var_refs: Vec<Arc<RefCell<VariableRef>>>,
    pub(crate) children: Vec<Arc<RefCell<Scope>>>,
    pub(crate) parent: Option<Weak<RefCell<Scope>>>,
    pub strict: bool,
}

impl Scope {
    pub fn is_child_of(&self, other: &Scope) -> bool {
        if self.node == other.node {
            return false;
        }
        other
            .node
            .text_range()
            .contains_range(self.node.text_range())
    }

    /// All of the variables available in this scope. This also includes variables which
    /// are inherited from outer scopes.
    pub fn variables(&self) -> impl Iterator<Item = &VariableBinding> {
        self.variables
            .iter()
            .map(|x| unsafe { borrow_unchecked(x.deref()) })
    }

    /// All of the variable references inside this scope. This also includes undefined references.
    pub fn var_refs(&self) -> impl Iterator<Item = &VariableRef> {
        self.var_refs
            .iter()
            .map(|x| unsafe { borrow_unchecked(x.deref()) })
    }

    /// All of the child scopes inside of this scope.
    pub fn children(&self) -> impl Iterator<Item = &Scope> {
        self.children
            .iter()
            .map(|x| unsafe { borrow_unchecked(x.deref()) })
    }

    /// The parent scope of this scope, if this scope is not a global scope.
    pub fn parent(&self) -> Option<Scope> {
        unsafe {
            self.parent.as_ref().map(|x| {
                borrow_unchecked(&Weak::upgrade(x).expect("scope dropped prematurely")).clone()
            })
        }
    }
}

/// How a variable was used in a reference to it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VariableUsageKind {
    /// Usages such as `foo.bar`, `let a = foo`, etc.
    Read,
    /// Calling the variable as a function, like `foo()`.
    Call,
    /// Constructing a new instance of the variable, like `new foo()`.
    Construct,
    /// Reassign the variable
    Write(Option<Expr>),
}

/// In what way a variable is declared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindingKind {
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
pub enum PatternBindingKind {
    /// A single identifier binding such as `let foo = 5`
    Literal,
    /// An object destructured binding such as `let { foo, bar } = foo`
    Object,
    /// An array destructured binding such as `let [ foo, bar ] = foo`
    Array,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportBindingKind {
    /// An import such as `import * as foo from "a"`
    NamedWildcard,
    /// An import from named imports, such as `import { foo, bar } from "a"`
    /// also includes the optional original name if this is an alias, aka `foo as bar`
    DestructuredImport(Option<SmolStr>),
    /// An import referencing a default, such as `import foo from "bar"`
    LiteralImport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScopeKind {
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
