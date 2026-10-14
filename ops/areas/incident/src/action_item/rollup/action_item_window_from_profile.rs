use super::*;

use std::collections::BTreeMap;

use crate::page::rule as page_rule;

pub fn summarize_page_profiles_for_window(items: &[page_rule::PageProfile]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_action_item_windows_from_profile(items: &[page_rule::PageProfile]) -> Vec<ActionItemWindow> {
    let mut out: Vec<ActionItemWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_action_item_window(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_action_item_window(item: &ActionItemWindow) -> String {
    page_rule::normalize_page_profile_code(&item.code)
}
