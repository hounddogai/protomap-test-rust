use super::*;

use std::collections::BTreeMap;

use crate::rollback::rollback_snapshot as rollback_rollback_snapshot;

pub fn summarize_capped_rollback_windows_for_assignment(
    items: &[rollback_rollback_snapshot::CappedRollbackWindow],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_approval_assignments_from_window(
    items: &[rollback_rollback_snapshot::CappedRollbackWindow],
) -> Vec<ApprovalAssignment> {
    let mut out: Vec<ApprovalAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_approval_assignment(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_approval_assignment(item: &ApprovalAssignment) -> String {
    rollback_rollback_snapshot::classify_rolling_rollback_snapshot(item.weighted_volume())
}
