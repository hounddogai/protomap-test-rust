use super::*;

use std::collections::BTreeMap;

use crate::dashboard::estimate as dashboard_estimate;

pub fn summarize_seasonal_dashboard_snapshots_for_estimate(
    items: &[dashboard_estimate::SeasonalDashboardSnapshot],
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

pub fn derive_pending_silence_estimates_from_snapshot(
    items: &[dashboard_estimate::SeasonalDashboardSnapshot],
) -> Vec<PendingSilenceEstimate> {
    let mut out: Vec<PendingSilenceEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_silence_estimate(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_pending_silence_estimate(item: &PendingSilenceEstimate) -> String {
    dashboard_estimate::default_dashboard_estimate_bucket(item.measured_value())
}
