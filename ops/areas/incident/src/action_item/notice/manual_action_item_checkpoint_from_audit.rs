use super::*;

use std::collections::BTreeMap;

use crate::timeline::baseline as timeline_baseline;

pub fn summarize_timeline_audits_for_checkpoint(items: &[timeline_baseline::TimelineAudit]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_manual_action_item_checkpoints_from_audit(
    items: &[timeline_baseline::TimelineAudit],
) -> Vec<ManualActionItemCheckpoint> {
    let mut out: Vec<ManualActionItemCheckpoint> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_action_item_checkpoint(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_manual_action_item_checkpoint(item: &ManualActionItemCheckpoint) -> String {
    timeline_baseline::classify_timeline_variance(item.adjusted_total())
}
