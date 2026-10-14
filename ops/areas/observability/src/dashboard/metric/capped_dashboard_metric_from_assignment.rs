use super::*;

use std::collections::BTreeMap;

use crate::slo::assignment as slo_assignment;

pub fn summarize_batched_slo_assignments_for_metric(
    items: &[slo_assignment::BatchedSloAssignment],
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

pub fn derive_capped_dashboard_metrics_from_assignment(
    items: &[slo_assignment::BatchedSloAssignment],
) -> Vec<CappedDashboardMetric> {
    let mut out: Vec<CappedDashboardMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_dashboard_metric(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_capped_dashboard_metric(item: &CappedDashboardMetric) -> f64 {
    slo_assignment::primary_slo_forecast_weight(3, true)
}
