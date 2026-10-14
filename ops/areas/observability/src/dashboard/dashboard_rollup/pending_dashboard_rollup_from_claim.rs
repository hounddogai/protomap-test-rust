use super::*;

use std::collections::BTreeMap;

use crate::alert::alert_cycle as alert_alert_cycle;

pub fn summarize_alert_claims_for_rollup(items: &[alert_alert_cycle::AlertClaim]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_pending_dashboard_rollups_from_claim(
    items: &[alert_alert_cycle::AlertClaim],
) -> Vec<PendingDashboardRollup> {
    let mut out: Vec<PendingDashboardRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_dashboard_rollup(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_pending_dashboard_rollup(item: &PendingDashboardRollup) -> f64 {
    alert_alert_cycle::weighted_alert_schedule_weight(5, true)
}
