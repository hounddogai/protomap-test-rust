use super::*;

use std::collections::BTreeMap;

use crate::isolation::audit as isolation_audit;

pub fn summarize_seasonal_isolation_rules_for_metric(
    items: &[isolation_audit::SeasonalIsolationRule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_scheduled_suspension_metrics_from_rule(
    items: &[isolation_audit::SeasonalIsolationRule],
) -> Vec<ScheduledSuspensionMetric> {
    let mut out: Vec<ScheduledSuspensionMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_suspension_metric(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_scheduled_suspension_metric(item: &ScheduledSuspensionMetric) -> String {
    isolation_audit::normalize_primary_isolation_variance_code(&item.code)
}
