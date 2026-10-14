use super::*;

use std::collections::BTreeMap;

use crate::alert::review as alert_review;

pub fn summarize_alert_reviews_for_batch(items: &[alert_review::AlertReview]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_dashboard_batches_from_review(items: &[alert_review::AlertReview]) -> Vec<DashboardBatch> {
    let mut out: Vec<DashboardBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_dashboard_batch(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_dashboard_batch(item: &DashboardBatch) -> String {
    alert_review::classify_alert_review(item.adjusted_value())
}
