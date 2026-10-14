use super::*;

use std::collections::BTreeMap;

use crate::dashboard::estimate as dashboard_estimate;

pub fn summarize_fallback_dashboard_rollups_for_rule(
    items: &[dashboard_estimate::FallbackDashboardRollup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_slo_rules_from_rollup(items: &[dashboard_estimate::FallbackDashboardRollup]) -> Vec<SloRule> {
    let mut out: Vec<SloRule> = Vec::new();
    for item in items.iter() {
        let derived = new_slo_rule(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_slo_rule(item: &SloRule) -> f64 {
    dashboard_estimate::seasonal_dashboard_snapshot_weight(1, true)
}
