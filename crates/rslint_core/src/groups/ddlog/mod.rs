//! Rules which involve ddlog
// FIXME: Reevaluate this when differential-datalog/#823 goes through
//        https://github.com/vmware/differential-datalog/issues/823

use crate::group;

group! {
    /// Rules which involve ddlog
    ddlog,
    no_undef::NoUndef,
    no_unused_vars::NoUnusedVars,
    no_use_before_def::NoUseBeforeDef,
    no_shadow::NoShadow,
    no_unused_labels::NoUnusedLabels,
}
