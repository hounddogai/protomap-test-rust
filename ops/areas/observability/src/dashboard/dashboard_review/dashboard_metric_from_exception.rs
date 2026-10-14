use super::*;

use std::collections::BTreeMap;

use crate::dashboard::review as dashboard_review;

pub fn summarize_dashboard_exceptions_for_metric(
    items: &[dashboard_review::DashboardException],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_dashboard_metrics_from_exception(items: &[dashboard_review::DashboardException]) -> Vec<DashboardMetric> {
    let mut out: Vec<DashboardMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_dashboard_metric(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_dashboard_metric(item: &DashboardMetric) -> String {
    dashboard_review::classify_lenient_dashboard_review(item.projected_load())
}
