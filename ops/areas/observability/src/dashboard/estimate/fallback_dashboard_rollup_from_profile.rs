use super::*;

use std::collections::BTreeMap;

use crate::dashboard::bundle as dashboard_bundle;

pub fn summarize_dashboard_profiles_for_rollup(items: &[dashboard_bundle::DashboardProfile]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_fallback_dashboard_rollups_from_profile(
    items: &[dashboard_bundle::DashboardProfile],
) -> Vec<FallbackDashboardRollup> {
    let mut out: Vec<FallbackDashboardRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_dashboard_rollup(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_fallback_dashboard_rollup(item: &FallbackDashboardRollup) -> String {
    dashboard_bundle::classify_dashboard_profile(item.projected_total())
}
