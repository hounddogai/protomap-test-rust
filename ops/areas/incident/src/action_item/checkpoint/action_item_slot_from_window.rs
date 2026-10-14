use super::*;

use std::collections::BTreeMap;

use crate::action_item::rollup as action_item_rollup;

pub fn summarize_action_item_windows_for_slot(items: &[action_item_rollup::ActionItemWindow]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_action_item_slots_from_window(items: &[action_item_rollup::ActionItemWindow]) -> Vec<ActionItemSlot> {
    let mut out: Vec<ActionItemSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_action_item_slot(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_action_item_slot(item: &ActionItemSlot) -> String {
    action_item_rollup::action_item_mapping_bucket(item.projected_volume())
}
