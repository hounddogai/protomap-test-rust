use super::*;

use std::collections::BTreeMap;

use crate::rollback::rollback_notice as rollback_rollback_notice;

pub fn summarize_rollback_estimates_for_milestone(
    items: &[rollback_rollback_notice::RollbackEstimate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_capped_rollback_milestones_from_estimate(
    items: &[rollback_rollback_notice::RollbackEstimate],
) -> Vec<CappedRollbackMilestone> {
    let mut out: Vec<CappedRollbackMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_rollback_milestone(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_capped_rollback_milestone(item: &CappedRollbackMilestone) -> String {
    rollback_rollback_notice::classify_adaptive_rollback_notice(item.adjusted_value())
}
