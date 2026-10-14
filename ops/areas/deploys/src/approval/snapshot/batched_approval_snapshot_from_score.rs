use super::*;

use std::collections::BTreeMap;

use crate::rollback::rollback_draft as rollback_rollback_draft;

pub fn summarize_rollback_scores_for_snapshot(
    items: &[rollback_rollback_draft::RollbackScore],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_batched_approval_snapshots_from_score(
    items: &[rollback_rollback_draft::RollbackScore],
) -> Vec<BatchedApprovalSnapshot> {
    let mut out: Vec<BatchedApprovalSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_approval_snapshot(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_batched_approval_snapshot(item: &BatchedApprovalSnapshot) -> String {
    rollback_rollback_draft::normalize_global_rollback_draft_code(&item.code)
}
