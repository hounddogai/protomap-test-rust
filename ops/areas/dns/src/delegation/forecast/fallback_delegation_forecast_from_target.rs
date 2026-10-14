use super::*;

use std::collections::BTreeMap;

use crate::failover::failover_forecast as failover_failover_forecast;

pub fn summarize_failover_targets_for_forecast(
    items: &[failover_failover_forecast::FailoverTarget],
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

pub fn derive_fallback_delegation_forecasts_from_target(
    items: &[failover_failover_forecast::FailoverTarget],
) -> Vec<FallbackDelegationForecast> {
    let mut out: Vec<FallbackDelegationForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_delegation_forecast(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_fallback_delegation_forecast(item: &FallbackDelegationForecast) -> String {
    failover_failover_forecast::strict_failover_review_bucket(item.adjusted_volume())
}
