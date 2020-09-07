//! Core traits for rule definitions and rule context.
//! As well as an internal prelude to make imports for rules easier.

#![allow(unused_variables, unused_imports)]

use crate::{Diagnostic, DiagnosticBuilder};
use codespan_reporting::diagnostic::Severity;
use dyn_clone::DynClone;
use rslint_parser::{SyntaxNode, SyntaxNodeExt, SyntaxToken};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::fmt::Debug;
use std::marker::{Send, Sync};
use std::ops::{Deref, DerefMut, Drop};
use std::rc::Rc;

/// The main type of rule run by the runner. The rule takes individual
/// nodes inside of a Concrete Syntax Tree and checks them.
/// It may also take individual syntax tokens.
/// Rule must be all be [`Send`] + [`Sync`], because rules are run in parallel.
///
/// # Rule Level Configuration
/// Rules do not know about the lint level they were configured for, the runner
/// runs the rules, then maps any error/warning diagnostics to their appropriate severity.
/// This saves on boilerplate code for getting the appropriate diagnostic builder type and config.
///
/// # Guidelines
/// This is a list of guidelines and tips you should generally follow when implementing a rule:
/// - Do not use text based equality, it is inaccurate, instead use [`lexical_eq`](SyntaxNodeExt::lexical_eq).
/// - Avoid using `text_range` on nodes, it is inaccurate because it may include whitespace, instead use [`trimmed_range`](SyntaxNodeExt::trimmed_range).
/// - Avoid using `text` on nodes for the same reason as the previous, use [`trimmed_text`](SyntaxNodeExt::trimmed_text).
/// - If you can offer better diagnostics and more context around a rule error, __always__ do it! It is a central goal
/// of the project to offer very helpful diagnostics.
/// - Do not be afraid to clone syntax nodes, ast nodes, and syntax tokens. They are all backed by an [`Rc`](std::rc::Rc) around Node data.
/// therefore they can be cheaply cloned (but if you can, have your functions take a reference since Rc cloning is not zero cost).
/// - Do not try to rely on the result of other rules, it is impossible because rules are run at the same time.
/// - Do not rely on file data of different files. There is a separate rule type for this.
/// - Do not unwrap pieces of an AST node (sometimes it is ok because they are guaranteed to be there), since that will cause panics
/// with error recovery.
/// - Do not use node or string coloring outside of diagnostic nodes, it messes with termcolor and ends up looking horrible.
pub trait CstRule: Send + Sync + Rule {
    /// Check an individual node in the syntax tree.
    /// You can use the `match_ast` macro to make matching a node to an ast node easier.
    /// The reason this uses nodes and not a visitor is because nodes are more flexible,
    /// converting them to an AST node has zero cost and you can easily traverse surrounding nodes.
    /// Defaults to doing nothing.
    ///
    /// The return type is `Option<()>` to allow usage of `?` on the properties of AST nodes which are all optional.
    fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        None
    }

    /// Check an individual token in the syntax tree.
    /// Defaults to doing nothing.
    fn check_token(&self, token: &SyntaxToken, ctx: &mut RuleCtx) -> Option<()> {
        None
    }

    /// Check the root of the tree one time.
    /// This method is guaranteed to only be called once.
    /// The root's kind will be either `SCRIPT` or `MODULE`.
    /// Defaults to doing nothing.
    fn check_root(&self, root: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        None
    }
}

/// A generic trait which describes things common to a rule regardless on what they run on.
///
/// Each rule should have a `new` function for easy instantiation. We however do not require this
/// for the purposes of allowing more complex rules to instantiate themselves in a different way.
/// However the rules must be easily instantiated because of rule groups.
#[typetag::serde]
pub trait Rule: Debug + DynClone {
    /// A unique, kebab-case name for the rule.
    fn name(&self) -> &'static str;
    /// The name of the group this rule belongs to.
    fn group(&self) -> &'static str;
}

/// Context given to a rule when running it.
// This is passed by reference and not by Arc, which is very important,
// Arcs are very expensive to copy, and for 50 rules running on 50 files we will have a total of
// 2500 copies, which is non ideal at best.
#[derive(Debug, Clone)]
pub struct RuleCtx {
    /// The file id of the file being linted.
    pub file_id: usize,
    /// Whether the linter is run with the `--verbose` option.
    /// Which dictates whether the linter should include more (potentially spammy) context in diagnostics.
    pub verbose: bool,
    pub diagnostics: Vec<Diagnostic>,
}

impl RuleCtx {
    /// Make a new diagnostic builder. The diagnostic will automatically be added to the context
    /// once the guard is dropped.
    pub fn err(&mut self, code: impl AsRef<str>, message: impl AsRef<str>) -> DiagnosticBuilder {
        DiagnosticBuilder::error(self.file_id, code.as_ref(), message.as_ref())
    }

    pub fn add_err(&mut self, diagnostic: impl Into<Diagnostic>) {
        self.diagnostics.push(diagnostic.into())
    }
}

/// The result of running a single rule on a syntax tree. 
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuleResult {
    pub diagnostics: Vec<Diagnostic>,
}

impl RuleResult {
    /// Get the result of running this rule.
    pub fn outcome(&self) -> Outcome {
        Outcome::from(&self.diagnostics)
    }

    pub fn merge(self, other: RuleResult) -> RuleResult {
        RuleResult {
            diagnostics: [self.diagnostics, other.diagnostics].concat(),
        }
    }
}

/// The overall result of running a single rule or linting a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Outcome {
    /// Running the rule resulted in one or more errors.
    /// The rule result may have also included warnings or notes.
    Failure,
    /// Running the rule resulted in one or more warnings.
    /// May also include notes.
    Warning,
    /// Running the rule resulted in no errors or warnings.  
    /// May include note diagnostics (which are very rare).
    Success,
}

impl<T> From<T> for Outcome
where
    T: IntoIterator,
    T::Item: Borrow<Diagnostic>,
{
    fn from(diagnostics: T) -> Self {
        let mut outcome = Outcome::Success;
        for diagnostic in diagnostics {
            match diagnostic.borrow().severity {
                Severity::Error | Severity::Bug => outcome = Outcome::Failure,
                Severity::Warning if outcome != Outcome::Failure => outcome = Outcome::Warning,
                _ => {}
            }
        }
        outcome
    }
}

impl Outcome {
    pub fn merge(outcomes: impl IntoIterator<Item = impl Borrow<Outcome>>) -> Outcome {
        let mut overall = Outcome::Success;
        for outcome in outcomes {
            match outcome.borrow() {
                Outcome::Failure => overall = Outcome::Failure,
                Outcome::Warning if overall != Outcome::Failure => overall = Outcome::Warning,
                _ => {}
            }
        }
        overall
    }
}

/// A macro to easily generate rule boilerplate code.
///
/// ```ignore
/// declare_lint! {
///     /// A description of the rule here
///     /// This will be used as the doc for the rule struct
///     RuleName,
///     // The name of the group this rule belongs to. 
///     groupname,
///     // Make sure this is kebab-case and unique.
///     "rule-name",
///     /// A description of the attribute here, used for config docs.
///     pub config_attr: u8,
///     pub another_attr: String
/// }
/// ```
///
/// # Rule name and docs
///
/// The macro's first argument is an identifier for the rule structure.
/// This should always be a PascalCase name. You will have to either derive Default for the struct
/// or implement it manually.
///
/// The macro also accepts any doc comments for the rule name. These comments
/// are then used by an xtask script to generate markdown files for user facing docs.
/// Each rule doc should include an `Incorrect Code Examples` header. It may also optionally
/// include a `Correct Code Examples`. Do not include a `Config` header, it is autogenerated
/// from config field docs.
///
/// # Config
///
/// After the rule code, the macro accepts fields for the struct. Any field which is
/// public will be used for config, you can however disable this by using `#[serde(skip)]`.
/// Every public (config) field should have a doc comment, the doc comments will be used for
/// user facing documentation. Therefore try to be non technical and non rust specific with the doc comments.
///
///
/// This will generate a rule struct with `RuleName`,
/// and use the optional config attributes defined for the config of the rule.
/// You must make sure each config field is Deserializable.
#[macro_export]
macro_rules! declare_lint {
    (
        $(#[$outer:meta])*
        // The rule struct name
        $name:ident,
        $group:ident,
        // A unique kebab-case name for the rule
        $code:expr
        $(,
            // Any fields for the rule
            $(
                $(#[$inner:meta])*
                $visibility:vis $key:ident : $val:ty
            ),*
        )?
    ) => {
        use $crate::Rule;
        use serde::{Deserialize, Serialize};

        $(#[$outer])*
        #[derive(Debug, Clone, Deserialize, Serialize)]
        pub struct $name {
            $(
                $(
                    $(#[$inner])*
                    pub $key: $val
                ),
            *)?
        }

        impl $name {
            pub fn new() -> Self {
                Self::default()
            }
        }

        #[typetag::serde(name = $code)]
        impl Rule for $name {
            fn name(&self) -> &'static str {
                $code
            }

            fn group(&self) -> &'static str {
                stringify!($group)
            }
        }
    }
}
