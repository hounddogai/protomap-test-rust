use super::*;

use std::collections::BTreeMap;

use crate::identity::window as identity_window;

pub fn summarize_identity_windows_for_allocation(items: &[identity_window::IdentityWindow]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_identity_allocations_from_window(items: &[identity_window::IdentityWindow]) -> Vec<IdentityAllocation> {
    let mut out: Vec<IdentityAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_identity_allocation(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_identity_allocation(item: &IdentityAllocation) -> String {
    identity_window::classify_scheduled_identity_window(item.projected_total())
}
