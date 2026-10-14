use super::*;

use std::collections::BTreeMap;

use crate::allowlist::slot as allowlist_slot;

pub fn summarize_allowlist_slots_for_bundle(items: &[allowlist_slot::AllowlistSlot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_allowlist_bundles_from_slot(items: &[allowlist_slot::AllowlistSlot]) -> Vec<AllowlistBundle> {
    let mut out: Vec<AllowlistBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_allowlist_bundle(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}
