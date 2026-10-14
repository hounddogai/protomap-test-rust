use super::*;

use std::collections::BTreeMap;

use crate::rollback::manifest as rollback_manifest;

pub fn summarize_fallback_rollback_policies_for_checkpoint(
    items: &[rollback_manifest::FallbackRollbackPolicy],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_batched_approval_checkpoints_from_policy(
    items: &[rollback_manifest::FallbackRollbackPolicy],
) -> Vec<BatchedApprovalCheckpoint> {
    let mut out: Vec<BatchedApprovalCheckpoint> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_approval_checkpoint(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_batched_approval_checkpoint(item: &BatchedApprovalCheckpoint) -> f64 {
    rollback_manifest::fallback_rollback_policy_weight(9, true)
}
