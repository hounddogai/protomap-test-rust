use super::*;

use std::collections::BTreeMap;

use crate::freeze::exception as freeze_exception;

pub fn summarize_freeze_overrides_for_milestone(items: &[freeze_exception::FreezeOverride]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_rollback_milestones_from_override(items: &[freeze_exception::FreezeOverride]) -> Vec<RollbackMilestone> {
    let mut out: Vec<RollbackMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_rollback_milestone(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_rollback_milestone(item: &RollbackMilestone) -> String {
    freeze_exception::classify_freeze_exception(item.projected_total())
}
