use super::*;

use std::collections::BTreeMap;

use crate::alert::review as alert_review;

pub fn summarize_alert_reviews_for_target(items: &[alert_review::AlertReview]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_slo_targets_from_review(items: &[alert_review::AlertReview]) -> Vec<SloTarget> {
    let mut out: Vec<SloTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_slo_target(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_slo_target(item: &SloTarget) -> f64 {
    alert_review::alert_policy_weight(1, true)
}
