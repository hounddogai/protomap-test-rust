use super::*;

use std::collections::BTreeMap;

use crate::slo::window as slo_window;

pub fn summarize_slo_allocations_for_metric(items: &[slo_window::SloAllocation]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_dashboard_metrics_from_allocation(items: &[slo_window::SloAllocation]) -> Vec<DashboardMetric> {
    let mut out: Vec<DashboardMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_dashboard_metric(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_dashboard_metric(item: &DashboardMetric) -> String {
    slo_window::slo_window_bucket(item.projected_load())
}
