use internment::Intern;
use once_cell::sync::Lazy;
use types__ast::{Pattern, Span, Spanned};

/// The implicitly introduced `arguments` variable for function scopes,
/// kept in a global so we only allocate & intern it once
pub static IMPLICIT_ARGUMENTS: Lazy<Intern<Pattern>> = Lazy::new(|| {
    Intern::new(Pattern::SinglePattern {
        name: Some(Spanned {
            data: Intern::new("arguments".to_owned()),
            // TODO: Give this the span of the creating function I guess
            span: Span::new(0, 0),
        })
        .into(),
    })
});
