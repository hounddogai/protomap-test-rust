use super::*;

use std::collections::BTreeMap;

use crate::ttl_policy::estimate as ttl_policy_estimate;

pub fn summarize_ttl_policy_forecasts_for_assignment(
    items: &[ttl_policy_estimate::TtlPolicyForecast],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_batched_health_check_assignments_from_forecast(
    items: &[ttl_policy_estimate::TtlPolicyForecast],
) -> Vec<BatchedHealthCheckAssignment> {
    let mut out: Vec<BatchedHealthCheckAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_health_check_assignment(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_batched_health_check_assignment(item: &BatchedHealthCheckAssignment) -> String {
    ttl_policy_estimate::ttl_policy_forecast_bucket(item.projected_volume())
}
