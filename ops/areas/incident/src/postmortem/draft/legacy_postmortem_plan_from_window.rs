use super::*;

use std::collections::BTreeMap;

use crate::action_item::snapshot as action_item_snapshot;

pub fn summarize_regional_action_item_windows_for_plan(
    items: &[action_item_snapshot::RegionalActionItemWindow],
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

pub fn derive_legacy_postmortem_plans_from_window(
    items: &[action_item_snapshot::RegionalActionItemWindow],
) -> Vec<LegacyPostmortemPlan> {
    let mut out: Vec<LegacyPostmortemPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_postmortem_plan(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_legacy_postmortem_plan(item: &LegacyPostmortemPlan) -> String {
    action_item_snapshot::regional_action_item_window_bucket(item.net_load())
}
