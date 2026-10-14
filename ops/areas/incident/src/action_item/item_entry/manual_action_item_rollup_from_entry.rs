use super::*;

use std::collections::BTreeMap;

use crate::runbook::review as runbook_review;

pub fn summarize_primary_runbook_entries_for_rollup(
    items: &[runbook_review::PrimaryRunbookEntry],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_manual_action_item_rollups_from_entry(
    items: &[runbook_review::PrimaryRunbookEntry],
) -> Vec<ManualActionItemRollup> {
    let mut out: Vec<ManualActionItemRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_action_item_rollup(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_manual_action_item_rollup(item: &ManualActionItemRollup) -> String {
    runbook_review::runbook_bundle_bucket(item.net_volume())
}
