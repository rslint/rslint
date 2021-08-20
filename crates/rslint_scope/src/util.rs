use rslint_parser::{
    ast::{Expr, Name, ObjectPatternProp, Pattern},
    *,
};

#[inline]
pub(crate) fn expand_pattern(
    pat: Pattern,
    func: &mut impl FnMut(Name),
    expr_handler: &mut impl FnMut(Expr),
) {
    match pat {
        Pattern::ArrayPattern(arr_pat) => {
            for item in arr_pat.elements() {
                expand_pattern(item, func, expr_handler);
            }
        }
        Pattern::AssignPattern(assign_pat) => {
            if let Some(key) = assign_pat.key() {
                expand_pattern(key, func, expr_handler);
            }
            if let Some(val) = assign_pat.value() {
                expr_handler(val);
            }
        }
        Pattern::ExprPattern(_) => {}
        Pattern::ObjectPattern(obj_pat) => {
            for elem in obj_pat.elements() {
                match elem {
                    ObjectPatternProp::AssignPattern(pat) => {
                        if let Some(key) = pat.key() {
                            expand_pattern(key, func, expr_handler);
                        }
                        if let Some(val) = pat.value() {
                            expr_handler(val);
                        }
                    }
                    ObjectPatternProp::KeyValuePattern(pat) => {
                        if let Some(val) = pat.value() {
                            expand_pattern(val, func, expr_handler);
                        }
                    }
                    ObjectPatternProp::RestPattern(pat) => {
                        if let Some(val) = pat.pat() {
                            expand_pattern(val, func, expr_handler);
                        }
                    }
                    ObjectPatternProp::SinglePattern(pat) => {
                        expand_pattern(Pattern::SinglePattern(pat), func, expr_handler);
                    }
                }
            }
        }
        Pattern::RestPattern(pat) => {
            if let Some(val) = pat.pat() {
                expand_pattern(val, func, expr_handler);
            }
        }
        Pattern::SinglePattern(pat) => {
            if let Some(name) = pat.name() {
                func(name);
            }
        }
    }
}
