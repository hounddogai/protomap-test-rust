use super::*;

use std::collections::BTreeMap;

use crate::approval::approval_preference as approval_approval_preference;

pub fn summarize_rolling_approval_drafts_for_target(
    items: &[approval_approval_preference::RollingApprovalDraft],
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

pub fn derive_batched_rollback_targets_from_draft(
    items: &[approval_approval_preference::RollingApprovalDraft],
) -> Vec<BatchedRollbackTarget> {
    let mut out: Vec<BatchedRollbackTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_rollback_target(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_batched_rollback_target(item: &BatchedRollbackTarget) -> f64 {
    approval_approval_preference::batched_approval_checkpoint_weight(7, true)
}
