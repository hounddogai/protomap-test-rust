use super::*;

use std::collections::BTreeMap;

use crate::measurement::record_item as measurement_record_item;

pub fn summarize_fallback_measurement_estimates_for_estimate(
    items: &[measurement_record_item::FallbackMeasurementEstimate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_default_policy_estimates_from_estimate(
    items: &[measurement_record_item::FallbackMeasurementEstimate],
) -> Vec<DefaultPolicyEstimate> {
    let mut out: Vec<DefaultPolicyEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_default_policy_estimate(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_default_policy_estimate(item: &DefaultPolicyEstimate) -> String {
    measurement_record_item::normalize_tiered_measurement_record_code(&item.code)
}
