use super::*;

use std::collections::BTreeMap;

use crate::dashboard::dashboard_review as dashboard_dashboard_review;

pub fn summarize_dashboard_metrics_for_profile(
    items: &[dashboard_dashboard_review::DashboardMetric],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_slo_profiles_from_metric(items: &[dashboard_dashboard_review::DashboardMetric]) -> Vec<SloProfile> {
    let mut out: Vec<SloProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_slo_profile(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_slo_profile(item: &SloProfile) -> String {
    dashboard_dashboard_review::capped_dashboard_review_bucket(item.net_value())
}
