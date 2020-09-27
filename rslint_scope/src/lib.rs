//! A scope analysis library for JavaScript which strives to be as detailed as possible.

mod analyzer;

use analyzer::*;
use rslint_parser::{parse_module, SmolStr, SyntaxKind, SyntaxNode, SyntaxText};
use std::rc::{Rc, Weak};

/// A variable which exists in a particular scope.
#[derive(Debug, Clone)]
pub struct VariableBinding {
    /// The node which defined this variable
    /// `None` if the variable is a global.
    pub node: Option<SyntaxNode>,
    pub name: SmolStr,
    /// All the references to this variable
    pub references: Vec<Weak<VariableRef>>,
    /// A stack of the variable definitions this definition shadows
    pub shadows: Vec<Weak<VariableBinding>>,
    pub kind: BindingKind,
    /// Whether this variable is not defined in this scope but is instead passed down
    /// from a parent scope
    pub inherited: bool,
}

#[derive(Debug, Clone)]
pub struct VariableRef {
    /// The node which houses the variable reference, e.g. `foo` in `foo + bar`.
    pub node: SyntaxNode,
    /// How the variable was used.
    pub usage: VariableUsageKind,
    /// The variable declaration, if the variable was actually defined.
    pub declaration: Option<Weak<VariableBinding>>,
    /// The possible variable declaration if its a let or const binding and the variable ref
    /// is in a temporal dead zone. Aka:
    ///
    /// ```js
    /// foo + 5; // temporal dead zone
    ///
    /// let foo = 5;
    /// ```
    pub hoisted_declaration: Option<Weak<VariableBinding>>,
}

impl VariableRef {
    pub fn undefined(&self) -> bool {
        self.declaration.is_none()
    }

    pub fn in_temporal_dead_zone(&self) -> bool {
        self.hoisted_declaration.is_none()
    }
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub node: SyntaxNode,
    pub kind: ScopeKind,
    pub variables: Vec<Rc<VariableBinding>>,
    pub var_refs: Vec<Rc<VariableRef>>,
    pub children: Vec<Rc<Scope>>,
    pub parent: Option<Weak<Scope>>,
    pub unreachable: bool,
}

/// How a variable was used in a reference to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VariableUsageKind {
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
}

#[test]
fn it_works() {
    let src = r#"
    let foo = bar;

    let ee = {
        foo,
        get baz() {
            if (foo > 5) {
                bar();
            }
        }
    }
    var bar = 5;
    "#;
    let mut analyzer = Analyzer::new_root(parse_module(src, 0).syntax());
    analyzer.analyze(None);
    let global = analyzer.end();
    assert!(!global.children[0].children[0].var_refs[0].undefined());
    assert_eq!(
        global.children[0].children[0].var_refs[0].usage,
        VariableUsageKind::Call
    );
}
