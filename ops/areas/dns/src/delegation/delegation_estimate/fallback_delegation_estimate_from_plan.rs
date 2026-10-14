use super::*;

use std::collections::BTreeMap;

use crate::zone::forecast as zone_forecast;

pub fn summarize_tiered_zone_plans_for_estimate(items: &[zone_forecast::TieredZonePlan]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_fallback_delegation_estimates_from_plan(
    items: &[zone_forecast::TieredZonePlan],
) -> Vec<FallbackDelegationEstimate> {
    let mut out: Vec<FallbackDelegationEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_delegation_estimate(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_fallback_delegation_estimate(item: &FallbackDelegationEstimate) -> String {
    zone_forecast::primary_zone_metric_bucket(item.measured_value())
}
