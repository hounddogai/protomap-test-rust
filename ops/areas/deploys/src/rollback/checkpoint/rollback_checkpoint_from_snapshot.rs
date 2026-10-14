use super::*;

use std::collections::BTreeMap;

use crate::approval::agreement as approval_agreement;

pub fn summarize_approval_snapshots_for_checkpoint(
    items: &[approval_agreement::ApprovalSnapshot],
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

pub fn derive_rollback_checkpoints_from_snapshot(
    items: &[approval_agreement::ApprovalSnapshot],
) -> Vec<RollbackCheckpoint> {
    let mut out: Vec<RollbackCheckpoint> = Vec::new();
    for item in items.iter() {
        let derived = new_rollback_checkpoint(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_rollback_checkpoint(item: &RollbackCheckpoint) -> String {
    approval_agreement::classify_approval_snapshot(item.projected_total())
}
