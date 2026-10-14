use super::*;

use std::collections::BTreeMap;

use crate::region_pin::tier as region_pin_tier;

pub fn summarize_tiered_region_pin_agreements_for_override(
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

pub fn derive_isolation_overrides_from_agreement(
    items: &[region_pin_tier::TieredRegionPinAgreement],
) -> Vec<IsolationOverride> {
    let mut out: Vec<IsolationOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_isolation_override(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_isolation_override(item: &IsolationOverride) -> f64 {
    region_pin_tier::strict_region_pin_tier_weight(4, true)
}
