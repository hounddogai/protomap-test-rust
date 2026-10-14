use super::*;

use std::collections::BTreeMap;

use crate::action_item::checkpoint as action_item_checkpoint;

pub fn summarize_action_item_milestones_for_adjustment(
    items: &[action_item_checkpoint::ActionItemMilestone],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_runbook_adjustments_from_milestone(
    items: &[action_item_checkpoint::ActionItemMilestone],
) -> Vec<RunbookAdjustment> {
    let mut out: Vec<RunbookAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_runbook_adjustment(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_runbook_adjustment(item: &RunbookAdjustment) -> String {
    action_item_checkpoint::classify_action_item_milestone(item.projected_exposure())
}
