use super::*;

use std::collections::BTreeMap;

use crate::policy::policy_item as policy_policy_item;

pub fn summarize_pending_policy_overrides_for_metric(
    items: &[policy_policy_item::PendingPolicyOverride],
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

pub fn derive_weighted_measurement_metrics_from_override(
    items: &[policy_policy_item::PendingPolicyOverride],
) -> Vec<WeightedMeasurementMetric> {
    let mut out: Vec<WeightedMeasurementMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_measurement_metric(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_weighted_measurement_metric(item: &WeightedMeasurementMetric) -> String {
    policy_policy_item::normalize_pending_policy_override_code(&item.code)
}
