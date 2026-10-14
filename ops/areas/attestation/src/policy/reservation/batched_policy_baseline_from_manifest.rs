use super::*;

use std::collections::BTreeMap;

use crate::measurement::profile as measurement_profile;

pub fn summarize_seasonal_measurement_manifests_for_baseline(
    items: &[measurement_profile::SeasonalMeasurementManifest],
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

pub fn derive_batched_policy_baselines_from_manifest(
    items: &[measurement_profile::SeasonalMeasurementManifest],
) -> Vec<BatchedPolicyBaseline> {
    let mut out: Vec<BatchedPolicyBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_policy_baseline(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_batched_policy_baseline(item: &BatchedPolicyBaseline) -> String {
    measurement_profile::classify_measurement_cycle(item.measured_value())
}
