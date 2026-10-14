use super::*;

use std::collections::BTreeMap;

use crate::isolation::exception as isolation_exception;

pub fn summarize_scheduled_isolation_exceptions_for_estimate(
    items: &[isolation_exception::ScheduledIsolationException],
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

pub fn derive_seasonal_isolation_estimates_from_exception(
    items: &[isolation_exception::ScheduledIsolationException],
) -> Vec<SeasonalIsolationEstimate> {
    let mut out: Vec<SeasonalIsolationEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_isolation_estimate(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_seasonal_isolation_estimate(item: &SeasonalIsolationEstimate) -> String {
    isolation_exception::classify_isolation_mapping(item.adjusted_volume())
}
