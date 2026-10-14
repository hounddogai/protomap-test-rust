use super::*;

use std::collections::BTreeMap;

use crate::measurement::record_item as measurement_record_item;

pub fn summarize_fallback_measurement_estimates_for_rule(
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

pub fn derive_primary_measurement_rules_from_estimate(
    items: &[measurement_record_item::FallbackMeasurementEstimate],
) -> Vec<PrimaryMeasurementRule> {
    let mut out: Vec<PrimaryMeasurementRule> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_measurement_rule(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_primary_measurement_rule(item: &PrimaryMeasurementRule) -> String {
    measurement_record_item::normalize_tiered_measurement_record_code(&item.code)
}
