use super::*;

use std::collections::BTreeMap;

use crate::suspension::bundle as suspension_bundle;

pub fn summarize_suspension_claims_for_window(items: &[suspension_bundle::SuspensionClaim]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_tiered_region_pin_windows_from_claim(
    items: &[suspension_bundle::SuspensionClaim],
) -> Vec<TieredRegionPinWindow> {
    let mut out: Vec<TieredRegionPinWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_region_pin_window(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_tiered_region_pin_window(item: &TieredRegionPinWindow) -> f64 {
    suspension_bundle::strict_suspension_bundle_weight(4, true)
}
