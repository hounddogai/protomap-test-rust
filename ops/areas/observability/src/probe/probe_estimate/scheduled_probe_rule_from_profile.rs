use super::*;

use std::collections::BTreeMap;

use crate::dashboard::metric as dashboard_metric;

pub fn summarize_weighted_dashboard_profiles_for_rule(
    items: &[dashboard_metric::WeightedDashboardProfile],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_scheduled_probe_rules_from_profile(
    items: &[dashboard_metric::WeightedDashboardProfile],
) -> Vec<ScheduledProbeRule> {
    let mut out: Vec<ScheduledProbeRule> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_probe_rule(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_scheduled_probe_rule(item: &ScheduledProbeRule) -> String {
    dashboard_metric::normalize_weighted_dashboard_profile_code(&item.code)
}
