use super::*;

use std::collections::BTreeMap;

use crate::region_pin::request as region_pin_request;

pub fn summarize_regional_region_pin_requests_for_mapping(
    items: &[region_pin_request::RegionalRegionPinRequest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_fallback_region_pin_mappings_from_request(
    items: &[region_pin_request::RegionalRegionPinRequest],
) -> Vec<FallbackRegionPinMapping> {
    let mut out: Vec<FallbackRegionPinMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_region_pin_mapping(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_fallback_region_pin_mapping(item: &FallbackRegionPinMapping) -> String {
    region_pin_request::classify_weighted_region_pin_profile(item.projected_total())
}
