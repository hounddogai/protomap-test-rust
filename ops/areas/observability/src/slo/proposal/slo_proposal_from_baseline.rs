use super::*;

use std::collections::BTreeMap;

use crate::dashboard::window as dashboard_window;

pub fn summarize_pending_dashboard_baselines_for_proposal(
    items: &[dashboard_window::PendingDashboardBaseline],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_slo_proposals_from_baseline(items: &[dashboard_window::PendingDashboardBaseline]) -> Vec<SloProposal> {
    let mut out: Vec<SloProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_slo_proposal(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_slo_proposal(item: &SloProposal) -> f64 {
    dashboard_window::dashboard_estimate_weight(2, true)
}
