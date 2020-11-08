use internment::Intern;
use once_cell::sync::Lazy;

/// The implicitly introduced `arguments` variable for function scopes,
/// kept in a global so we only allocate & intern it once
pub static IMPLICIT_ARGUMENTS: Lazy<Intern<ast::Pattern>> = Lazy::new(|| {
    Intern::new(ast::Pattern::SinglePattern {
        name: Some(Intern::new("arguments".to_owned())).into(),
    })
});
