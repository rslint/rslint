use std::collections::HashMap;
use std::ops::{Deref, DerefMut, Range};

use crate::{CompletedMarker, Parser, SyntaxKind};

/// State kept by the parser while parsing. 
/// It is required for things such as strict mode or async functions
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserState {
    /// Whether `in` should be counted in a binary expression
    /// this is for `for...in` statements to prevent ambiguity.
    pub include_in: bool,
    /// Whether the parser is in an iteration statement and `continue` is allowed.
    pub continue_allowed: bool,
    /// Whether the parser is in an iteration or switch statement and
    /// `break` is allowed.
    pub break_allowed: bool,
    /// A list of labels for labelled statements used to report undefined label errors
    /// for break and continue, as well as duplicate labels
    pub labels: HashMap<String, Range<usize>>,
    /// Whether the parser is in a generator function like `function* a() {}`
    pub in_generator: bool,
    /// Whether the parser is inside of a function 
    pub in_function: bool,
    /// Whether we potentially are in a place to parse an arrow expression
    pub potential_arrow_start: bool,
    /// Whether we are in an async function
    pub in_async: bool,
    /// Whether we are in strict mode code, this uses an option for error reporting and a bool for top level
    pub strict: (Option<Range<usize>>, bool),
    /// Whether the code we are parsing is a module
    pub is_module: bool,
    /// The exported default item, used for checking duplicate defaults
    pub default_item: Option<Range<usize>>,
}

impl Default for ParserState {
    fn default() -> Self {
        Self {
            include_in: true,
            continue_allowed: false,
            break_allowed: false,
            labels: HashMap::new(),
            in_generator: false,
            in_function: false,
            potential_arrow_start: false,
            in_async: false,
            strict: (None, true),
            is_module: false,
            default_item: None
        }
    }
}

impl ParserState {
    pub fn module() -> Self {
        Self {
            include_in: true,
            continue_allowed: false,
            break_allowed: false,
            labels: HashMap::new(),
            in_generator: false,
            in_function: false,
            potential_arrow_start: false,
            in_async: false,
            // using a null range is fine because module is checked for first
            strict: (Some(0..0), true),
            is_module: true,
            default_item: None
        }
    }

    /// Check for duplicate defaults and update state
    pub fn check_default(&mut self, p: &mut Parser, mut marker: CompletedMarker) -> CompletedMarker {
        // A default export is already present
        if let Some(range) = self.default_item.as_ref().filter(|_| self.is_module) {
            let err = p.err_builder("Illegal duplicate default export declarations")
                .secondary(range.to_owned(), "the module's default export is first defined here")
                .primary(marker.range(p), "multiple default exports are erroneous");
            
            p.error(err);
            marker.change_kind(p, SyntaxKind::ERROR);
        } else if self.is_module {
            self.default_item = Some(marker.range(p).into());
        }
        marker
    }

    pub fn iteration_stmt(&mut self, set: bool) {
        self.continue_allowed = set;
        self.break_allowed = set;
    }

    /// Turn on strict mode and issue a warning for redundant strict mode declarations
    pub fn strict(&mut self, p: &mut Parser, range: Range<usize>, top_level: bool) {
        if self.is_module {
            let err = p.warning_builder("Redundant strict mode declaration in module")
                .primary(range, "")
                .help("Note: modules are always in strict mode");

            p.error(err);
            return;
        }

        // Dont set strict mode if its already declared so we dont issue misleading errors
        if let (Some(existing_range), top_level) = self.strict.to_owned() {
            let warning_str = if top_level {
                "strict mode is globally declared here"
            } else {
                "strict mode is first declared here"
            };

            let err = p.warning_builder("Redundant strict mode declaration")
                .secondary(existing_range, warning_str)
                .primary(range, "this declaration is redundant");

            p.error(err);
        } else {
            self.strict = (Some(range), top_level);
        }
    }
}

impl<'t> Parser<'t> {
    pub fn with_state<'a>(&'a mut self, state: ParserState) -> StateGuard<'a, 't> {
        let original_state = self.state.clone();
        self.state = state;
        StateGuard {
            original_state,
            inner: self
        }
    }
}

pub struct StateGuard<'p, 't> {
    inner: &'p mut Parser<'t>,
    original_state: ParserState
}

impl<'p, 't> Deref for StateGuard<'p, 't> {
    type Target = Parser<'t>;

    fn deref(&self) -> &Parser<'t> {
        &self.inner
    }
}

impl<'p, 't> DerefMut for StateGuard<'p, 't> {
    fn deref_mut(&mut self) -> &mut Parser<'t> {
        &mut self.inner
    }
}

impl<'p, 't> Drop for StateGuard<'p, 't> {
    fn drop(&mut self) {
        std::mem::swap(&mut self.inner.state, &mut self.original_state);
    }
}
