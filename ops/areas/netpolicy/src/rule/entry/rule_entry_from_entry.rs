use super::*;

use std::collections::BTreeMap;

use crate::peering::window as peering_window;

pub fn summarize_peering_entries_for_entry(items: &[peering_window::PeeringEntry]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_rule_entries_from_entry(items: &[peering_window::PeeringEntry]) -> Vec<RuleEntry> {
    let mut out: Vec<RuleEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_rule_entry(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_rule_entry(item: &RuleEntry) -> String {
    peering_window::normalize_peering_window_code(&item.code)
}
