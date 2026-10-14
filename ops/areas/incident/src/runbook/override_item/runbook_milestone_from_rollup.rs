use super::*;

use std::collections::BTreeMap;

use crate::action_item::rollup as action_item_rollup;

pub fn summarize_action_item_rollups_for_milestone(
    items: &[action_item_rollup::ActionItemRollup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_runbook_milestones_from_rollup(items: &[action_item_rollup::ActionItemRollup]) -> Vec<RunbookMilestone> {
    let mut out: Vec<RunbookMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_runbook_milestone(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_runbook_milestone(item: &RunbookMilestone) -> String {
    action_item_rollup::classify_action_item_mapping(item.net_load())
}
