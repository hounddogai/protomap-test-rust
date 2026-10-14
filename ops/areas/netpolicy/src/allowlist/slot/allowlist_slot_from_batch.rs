use super::*;

use std::collections::BTreeMap;

use crate::allowlist::override_item as allowlist_override_item;

pub fn summarize_allowlist_batches_for_slot(
    items: &[allowlist_override_item::AllowlistBatch],
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

pub fn derive_allowlist_slots_from_batch(items: &[allowlist_override_item::AllowlistBatch]) -> Vec<AllowlistSlot> {
    let mut out: Vec<AllowlistSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_allowlist_slot(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_allowlist_slot(item: &AllowlistSlot) -> String {
    allowlist_override_item::classify_allowlist_batch(item.net_exposure())
}
