use super::*;

use std::collections::BTreeMap;

use crate::allowlist::entry as allowlist_entry;

pub fn summarize_allowlist_bundles_for_estimate(items: &[allowlist_entry::AllowlistBundle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_allowlist_estimates_from_bundle(items: &[allowlist_entry::AllowlistBundle]) -> Vec<AllowlistEstimate> {
    let mut out: Vec<AllowlistEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_allowlist_estimate(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_allowlist_estimate(item: &AllowlistEstimate) -> f64 {
    allowlist_entry::allowlist_entry_weight(3, true)
}
