use super::{Datalog, DatalogResult};
use differential_datalog::{ddval::DDValConvert, program::IdxId, DDlog};
use rslint_scoping_ddlog::Indexes;
use types::*;

macro_rules! derived_facts {
    ($($function_name:ident($arg_name:ident : $arg_ty:ty) -> $relation_type:ident from $index_name:ident),* $(,)?) => {
        impl Datalog {
            $(
                pub fn $function_name(&self, $arg_name: Option<$arg_ty>,) -> DatalogResult<Vec<$relation_type>> {
                    let ddlog = self.datalog.lock().expect("failed to lock ddlog instance");

                    let query = if let Some(arg) = $arg_name {
                        ddlog
                            .hddlog
                            .query_index(Indexes::$index_name as IdxId, arg.into_ddvalue())?
                    } else {
                        ddlog
                            .hddlog
                            .dump_index(Indexes::$index_name as IdxId)?
                    };

                    let result = query
                        .into_iter()
                        .map(|value| unsafe { $relation_type::from_ddvalue(value) })
                        .collect();

                    Ok(result)
                }
            )*
        }
    };
}

derived_facts! {
    variables_for_scope(scope: Scope) -> NameInScope from Index_VariablesForScope,
    invalid_name_uses(scope: Scope) -> InvalidNameUse from Index_InvalidNameUse,
    var_use_before_declaration(name: Name) -> VarUseBeforeDeclaration from Index_VarUseBeforeDeclaration,
}
