use super::*;

use std::collections::BTreeMap;

use crate::region_pin::tier as region_pin_tier;

pub fn summarize_tiered_region_pin_agreements_for_window(
    items: &[region_pin_tier::TieredRegionPinAgreement],
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

pub fn derive_region_pin_windows_from_agreement(
    items: &[region_pin_tier::TieredRegionPinAgreement],
) -> Vec<RegionPinWindow> {
    let mut out: Vec<RegionPinWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_region_pin_window(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_region_pin_window(item: &RegionPinWindow) -> f64 {
    region_pin_tier::tiered_region_pin_agreement_weight(8, true)
}
