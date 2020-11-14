use super::{Datalog, DatalogResult};
use dashmap::{mapref::entry::Entry, DashMap};
use differential_datalog::{
    ddval::DDValConvert,
    ddval::DDValue,
    program::{IdxId, RelId},
    DDlog, DeltaMap,
};
use rslint_scoping_ddlog::{relid2name, Indexes, Relations};
use std::{
    fs::File,
    io::Write,
    ops::Deref,
    sync::{Arc, Mutex},
};
use types::{
    ast::{FileId, ScopeId},
    ddlog_std::tuple2,
    NameInScope, NoUndef, TypeofUndef, UnusedVariables, UseBeforeDecl,
};

macro_rules! derived_facts {
    ($($function_name:ident($arg_name:ident : $arg_ty:ty) -> $relation_type:ident from $index_name:ident),* $(,)?) => {
        impl Datalog {
            $(
                pub fn $function_name(&self, $arg_name: Option<$arg_ty>,) -> DatalogResult<Vec<$relation_type>> {
                    let query = if let Some(arg) = $arg_name {
                        self
                            .hddlog
                            .query_index(Indexes::$index_name as IdxId, arg.into_ddvalue())?
                    } else {
                        self
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
    variables_for_scope(query: tuple2<FileId, ScopeId>) -> NameInScope from Index_VariablesForScope,
}

#[derive(Debug, Clone)]
pub struct Outputs {
    inner: Arc<InnerOutputs>,
}

impl Outputs {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(InnerOutputs::new()),
        }
    }
}

impl Default for Outputs {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for Outputs {
    type Target = InnerOutputs;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

macro_rules! outputs {
    ($($output_field:ident : $output_type:ident),* $(,)?) => {
        #[derive(Debug)]
        pub struct InnerOutputs {
            $(
                pub $output_field: DashMap<$output_type, isize>,
            )*
            output_file: Mutex<Option<File>>,
        }

        impl InnerOutputs {
            pub fn new() -> Self {
                Self {
                    $(
                        $output_field: DashMap::new(),
                    )*
                    output_file: Mutex::new(None),
                }
            }

            pub fn update(&self, relation: RelId, value: DDValue, weight: isize) {
                match relation {
                    $(
                        rel if rel == Relations::$output_type as RelId => {
                            let value: $output_type = unsafe {
                                <$output_type as DDValConvert>::from_ddvalue(value)
                            };

                            match self.$output_field.entry(value) {
                                Entry::Occupied(mut occupied) => {
                                    let should_remove = {
                                        let old_weight = occupied.get_mut();
                                        *old_weight += weight;

                                        *old_weight <= 0
                                    };

                                    if should_remove {
                                        occupied.remove();
                                    }
                                }

                                Entry::Vacant(vacant) => {
                                    vacant.insert(weight);
                                }
                            }
                        }
                    )*

                    // TODO: Add error logging
                    _ => {}
                }
            }

            pub fn clear(&self) {
                $(
                    self.$output_field.clear();
                )*
            }
        }

        impl Default for InnerOutputs {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

impl InnerOutputs {
    pub fn with_output_file(&self, file: File) {
        *self.output_file.lock().unwrap() = Some(file);
    }

    pub fn batch_update(&self, updates: DeltaMap<DDValue>) {
        {
            let mut file = self.output_file.lock().unwrap();
            if let Some(file) = &mut *file {
                for (rel, changes) in updates.iter() {
                    writeln!(file, "Changes to relation {}", relid2name(*rel).unwrap()).unwrap();

                    for (val, weight) in changes.iter() {
                        writeln!(file, ">> {} {:+}", val, weight).unwrap();
                    }

                    if !changes.is_empty() {
                        writeln!(file).unwrap();
                    }
                }
            }
        }

        for (relation, values) in updates {
            for (value, weight) in values {
                self.update(relation, value, weight);
            }
        }
    }
}

outputs! {
    typeof_undef: TypeofUndef,
    no_undef: NoUndef,
    use_before_decl: UseBeforeDecl,
    unused_variables: UnusedVariables,
}
