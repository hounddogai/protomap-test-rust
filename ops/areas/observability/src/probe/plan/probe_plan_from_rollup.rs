use super::*;

use std::collections::BTreeMap;

use crate::dashboard::dashboard_rollup as dashboard_dashboard_rollup;

pub fn summarize_pending_dashboard_rollups_for_plan(
    items: &[dashboard_dashboard_rollup::PendingDashboardRollup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_probe_plans_from_rollup(items: &[dashboard_dashboard_rollup::PendingDashboardRollup]) -> Vec<ProbePlan> {
    let mut out: Vec<ProbePlan> = Vec::new();
    for item in items.iter() {
        let derived = new_probe_plan(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_probe_plan(item: &ProbePlan) -> f64 {
    dashboard_dashboard_rollup::batched_dashboard_baseline_weight(7, true)
}
