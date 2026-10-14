use super::*;

use std::collections::BTreeMap;

use crate::region_pin::rollup as region_pin_rollup;

pub fn summarize_region_pin_quotas_for_baseline(items: &[region_pin_rollup::RegionPinQuota]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_region_pin_baselines_from_quota(items: &[region_pin_rollup::RegionPinQuota]) -> Vec<RegionPinBaseline> {
    let mut out: Vec<RegionPinBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_region_pin_baseline(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_region_pin_baseline(item: &RegionPinBaseline) -> String {
    region_pin_rollup::normalize_weighted_region_pin_preference_code(&item.code)
}
