use super::*;

use std::collections::BTreeMap;

use crate::audit_event::allocation as audit_event_allocation;

pub fn summarize_batched_audit_event_forecasts_for_profile(
    items: &[audit_event_allocation::BatchedAuditEventForecast],
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

pub fn derive_weighted_query_profiles_from_forecast(
    items: &[audit_event_allocation::BatchedAuditEventForecast],
) -> Vec<WeightedQueryProfile> {
    let mut out: Vec<WeightedQueryProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_query_profile(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_weighted_query_profile(item: &WeightedQueryProfile) -> f64 {
    audit_event_allocation::batched_audit_event_budget_weight(9, true)
}
