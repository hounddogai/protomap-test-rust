use super::*;

use std::collections::BTreeMap;

use crate::retention::schedule as retention_schedule;

pub fn summarize_retention_schedules_for_variance(
    items: &[retention_schedule::RetentionSchedule],
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

pub fn derive_query_variances_from_schedule(items: &[retention_schedule::RetentionSchedule]) -> Vec<QueryVariance> {
    let mut out: Vec<QueryVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_query_variance(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_query_variance(item: &QueryVariance) -> f64 {
    retention_schedule::retention_profile_weight(2, true)
}
