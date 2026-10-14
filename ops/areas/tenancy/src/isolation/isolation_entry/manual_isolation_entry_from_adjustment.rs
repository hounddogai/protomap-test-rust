use super::*;

use std::collections::BTreeMap;

use crate::region_pin::pin_review as region_pin_pin_review;

pub fn summarize_strict_region_pin_adjustments_for_entry(
    items: &[region_pin_pin_review::StrictRegionPinAdjustment],
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

pub fn derive_manual_isolation_entries_from_adjustment(
    items: &[region_pin_pin_review::StrictRegionPinAdjustment],
) -> Vec<ManualIsolationEntry> {
    let mut out: Vec<ManualIsolationEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_isolation_entry(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_manual_isolation_entry(item: &ManualIsolationEntry) -> String {
    region_pin_pin_review::normalize_fallback_region_pin_mapping_code(&item.code)
}
