use super::*;

use std::collections::BTreeMap;

use crate::action_item::item_forecast as action_item_item_forecast;

pub fn summarize_scheduled_action_item_rollups_for_snapshot(
    items: &[action_item_item_forecast::ScheduledActionItemRollup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_scheduled_page_snapshots_from_rollup(
    items: &[action_item_item_forecast::ScheduledActionItemRollup],
) -> Vec<ScheduledPageSnapshot> {
    let mut out: Vec<ScheduledPageSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_page_snapshot(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_scheduled_page_snapshot(item: &ScheduledPageSnapshot) -> f64 {
    action_item_item_forecast::pending_action_item_proposal_weight(4, true)
}
