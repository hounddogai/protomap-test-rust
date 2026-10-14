use super::*;

use std::collections::BTreeMap;

use crate::dashboard::dashboard_rule as dashboard_dashboard_rule;

pub fn summarize_dashboard_thresholds_for_review(
    items: &[dashboard_dashboard_rule::DashboardThreshold],
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

pub fn derive_slo_reviews_from_threshold(items: &[dashboard_dashboard_rule::DashboardThreshold]) -> Vec<SloReview> {
    let mut out: Vec<SloReview> = Vec::new();
    for item in items.iter() {
        let derived = new_slo_review(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_slo_review(item: &SloReview) -> String {
    dashboard_dashboard_rule::classify_seasonal_dashboard_rule(item.weighted_exposure())
}
