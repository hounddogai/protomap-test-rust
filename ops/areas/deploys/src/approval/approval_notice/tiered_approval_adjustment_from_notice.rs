use super::*;

use std::collections::BTreeMap;

use crate::rollback::rollback_notice as rollback_rollback_notice;

pub fn summarize_adaptive_rollback_notices_for_adjustment(
    items: &[rollback_rollback_notice::AdaptiveRollbackNotice],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_tiered_approval_adjustments_from_notice(
    items: &[rollback_rollback_notice::AdaptiveRollbackNotice],
) -> Vec<TieredApprovalAdjustment> {
    let mut out: Vec<TieredApprovalAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_approval_adjustment(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_tiered_approval_adjustment(item: &TieredApprovalAdjustment) -> String {
    rollback_rollback_notice::classify_adaptive_rollback_notice(item.projected_total())
}
