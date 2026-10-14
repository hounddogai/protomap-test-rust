use super::*;

use std::collections::BTreeMap;

use crate::action_item::item_baseline as action_item_item_baseline;

pub fn summarize_batched_action_item_drafts_for_rollup(
    items: &[action_item_item_baseline::BatchedActionItemDraft],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_legacy_timeline_rollups_from_draft(
    items: &[action_item_item_baseline::BatchedActionItemDraft],
) -> Vec<LegacyTimelineRollup> {
    let mut out: Vec<LegacyTimelineRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_timeline_rollup(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_legacy_timeline_rollup(item: &LegacyTimelineRollup) -> f64 {
    action_item_item_baseline::batched_action_item_draft_weight(9, true)
}
