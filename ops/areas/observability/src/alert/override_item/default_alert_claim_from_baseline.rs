use super::*;

use std::collections::BTreeMap;

use crate::dashboard::dashboard_rollup as dashboard_dashboard_rollup;

pub fn summarize_batched_dashboard_baselines_for_claim(
    items: &[dashboard_dashboard_rollup::BatchedDashboardBaseline],
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

pub fn derive_default_alert_claims_from_baseline(
    items: &[dashboard_dashboard_rollup::BatchedDashboardBaseline],
) -> Vec<DefaultAlertClaim> {
    let mut out: Vec<DefaultAlertClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_default_alert_claim(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_default_alert_claim(item: &DefaultAlertClaim) -> String {
    dashboard_dashboard_rollup::classify_batched_dashboard_baseline(item.effective_exposure())
}
