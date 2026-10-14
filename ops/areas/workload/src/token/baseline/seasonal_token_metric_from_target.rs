use super::*;

use std::collections::BTreeMap;

use crate::federation::targets as federation_targets;

pub fn summarize_federation_targets_for_metric(
    items: &[federation_targets::FederationTarget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_seasonal_token_metrics_from_target(
    items: &[federation_targets::FederationTarget],
) -> Vec<SeasonalTokenMetric> {
    let mut out: Vec<SeasonalTokenMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_token_metric(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_seasonal_token_metric(item: &SeasonalTokenMetric) -> String {
    federation_targets::federation_target_bucket(item.measured_value())
}
