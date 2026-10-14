use super::*;

use std::collections::BTreeMap;

use crate::measurement::measurement_target as measurement_measurement_target;

pub fn summarize_measurement_metrics_for_estimate(
    items: &[measurement_measurement_target::MeasurementMetric],
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

pub fn derive_global_nonce_estimates_from_metric(
    items: &[measurement_measurement_target::MeasurementMetric],
) -> Vec<GlobalNonceEstimate> {
    let mut out: Vec<GlobalNonceEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_global_nonce_estimate(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_global_nonce_estimate(item: &GlobalNonceEstimate) -> f64 {
    measurement_measurement_target::global_measurement_profile_weight(1, true)
}
