use super::{Datalog, DatalogResult};
use ast::{FileId, Name, ScopeId, Span};
use dashmap::{mapref::entry::Entry, DashMap};
use ddlog_std::tuple2;
use differential_datalog::{
    ddval::{DDValConvert, DDValue},
    program::{IdxId, RelId},
    DDlog, DeltaMap,
};
use no_unused_labels::NoUnusedLabels;
use rslint_scoping_ddlog::{relid2name, Indexes, Relations};
use std::{
    fs::File,
    io::Write,
    ops::Deref,
    sync::{Arc, Mutex},
};
use types::{
    name_in_scope::NameInScope,
    outputs::{
        no_shadow::NoShadow, no_undef::NoUndef, typeof_undef::TypeofUndef,
        unused_vars::UnusedVariables, use_before_def::UseBeforeDef,
    },
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
                        .map($relation_type::from_ddvalue)
                        .collect();

                    Ok(result)
                }
            )*
        }
    };
}

derived_facts! {
    variables_for_scope(query: tuple2<FileId, ScopeId>) -> NameInScope from name_in_scope_Index_VariablesForScope,
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
    ($($output_field:ident : $output_type:ident, $output_rel:ident),* $(,)?) => {
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
                        rel if rel == Relations::$output_rel as RelId => {
                            let value: $output_type = <$output_type as DDValConvert>::from_ddvalue(value);

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
        let span = tracing::info_span!("ddlog batch update");
        let _guard = span.enter();

        tracing::trace!(
            "received a batch update with {} changes",
            updates
                .iter()
                .map(|(_, changes)| changes.len())
                .sum::<usize>(),
        );

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
    no_typeof_undef: TypeofUndef, outputs_typeof_undef_TypeofUndef,
    no_undef: NoUndef, outputs_no_undef_NoUndef,
    use_before_def: UseBeforeDef, outputs_use_before_def_UseBeforeDef,
    no_unused_vars: UnusedVariables, outputs_unused_vars_UnusedVariables,
    no_shadow: NoShadow, outputs_no_shadow_NoShadow,
    no_unused_labels: NoUnusedLabels, outputs_no_unused_labels_NoUnusedLabels,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DatalogLint {
    NoUndef {
        var: Name,
        span: Span,
        file: FileId,
    },
    NoUnusedVars {
        var: Name,
        declared: Span,
        file: FileId,
    },
    TypeofUndef {
        whole_expr: Span,
        undefined_portion: Span,
        file: FileId,
    },
    UseBeforeDef {
        name: Name,
        used: Span,
        declared: Span,
        file: FileId,
    },
    NoShadow {
        variable: Name,
        original: Span,
        shadow: Span,
        implicit: bool,
        file: FileId,
    },
    NoUnusedLabels {
        label: Name,
        span: Span,
        file: FileId,
    },
}

impl DatalogLint {
    pub fn is_no_undef(&self) -> bool {
        matches!(self, Self::NoUndef { .. })
    }

    pub fn is_no_unused_vars(&self) -> bool {
        matches!(self, Self::NoUnusedVars { .. })
    }

    pub fn is_typeof_undef(&self) -> bool {
        matches!(self, Self::TypeofUndef { .. })
    }

    pub fn is_use_before_def(&self) -> bool {
        matches!(self, Self::UseBeforeDef { .. })
    }

    pub fn is_no_shadow(&self) -> bool {
        matches!(self, Self::NoShadow { .. })
    }

    pub fn is_no_unused_labels(&self) -> bool {
        matches!(self, Self::NoUnusedLabels { .. })
    }

    #[cfg(test)]
    pub(crate) fn no_undef(var: impl Into<Name>, span: std::ops::Range<u32>) -> Self {
        Self::NoUndef {
            var: var.into(),
            span: span.into(),
            file: FileId::new(0),
        }
    }

    #[cfg(test)]
    pub(crate) fn no_unused_vars(var: impl Into<Name>, declared: std::ops::Range<u32>) -> Self {
        Self::NoUnusedVars {
            var: var.into(),
            declared: declared.into(),
            file: FileId::new(0),
        }
    }

    #[cfg(test)]
    pub(crate) fn typeof_undef(
        whole_expr: std::ops::Range<u32>,
        undefined_portion: std::ops::Range<u32>,
    ) -> Self {
        Self::TypeofUndef {
            whole_expr: whole_expr.into(),
            undefined_portion: undefined_portion.into(),
            file: FileId::new(0),
        }
    }

    #[cfg(test)]
    pub(crate) fn use_before_def(
        name: impl Into<Name>,
        used: std::ops::Range<u32>,
        declared: std::ops::Range<u32>,
    ) -> Self {
        Self::UseBeforeDef {
            name: name.into(),
            used: used.into(),
            declared: declared.into(),
            file: FileId::new(0),
        }
    }

    #[cfg(test)]
    pub(crate) fn no_shadow(
        variable: impl Into<Name>,
        original: std::ops::Range<u32>,
        shadow: std::ops::Range<u32>,
        implicit: bool,
    ) -> Self {
        Self::NoShadow {
            variable: variable.into(),
            original: original.into(),
            shadow: shadow.into(),
            implicit,
            file: FileId::new(0),
        }
    }

    #[cfg(test)]
    pub(crate) fn unused_label(label: impl Into<Name>, span: std::ops::Range<u32>) -> Self {
        Self::NoUnusedLabels {
            label: label.into(),
            span: span.into(),
            file: FileId::new(0),
        }
    }

    #[cfg(test)]
    pub(crate) fn file_id_mut(&mut self) -> &mut FileId {
        match self {
            Self::NoUndef { file, .. } => file,
            Self::NoUnusedVars { file, .. } => file,
            Self::TypeofUndef { file, .. } => file,
            Self::UseBeforeDef { file, .. } => file,
            Self::NoShadow { file, .. } => file,
            Self::NoUnusedLabels { file, .. } => file,
        }
    }
}
