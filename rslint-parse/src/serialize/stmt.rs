use crate::lexer::token::*;
use crate::parser::cst::stmt::*;
use std::string::ToString;

impl Stmt {
    pub fn to_string(&self, source: &str) -> String {
        match self {
            Stmt::Variable(data) => {
                let decls = data
                    .declared
                    .iter()
                    .map(|decl| {
                        decl.value.as_ref().map_or_else(
                            || decl.name.span.content(source).to_string(),
                            |val| {
                                format!(
                                    "{} = {}",
                                    decl.name.span.content(source),
                                    val.to_string(source).trim()
                                )
                            },
                        )
                    })
                    .collect::<Vec<String>>();
                
                // Calculate the approximate size of the final string to avoid reallocating on each iteration
                let alloc_size = (decls.iter().map(|d| d.len()).sum::<usize>() + (data.declared.len() - 1) * 2) + 5;
                let mut ret = String::with_capacity(alloc_size);
                ret.push_str("var ");
                for (idx, decl) in decls.iter().enumerate() {
                    if idx == decls.len() - 1 {
                        ret.push_str(decl);
                    } else {
                        ret.push_str(decl);
                        ret.push_str(", ");
                    }
                }
                ret.push_str(";");
                ret
            }
        }
    }
}
