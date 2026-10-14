use super::*;

use std::collections::BTreeMap;

use crate::probe::probe_estimate as probe_probe_estimate;

pub fn summarize_scheduled_probe_rules_for_notice(
    items: &[probe_probe_estimate::ScheduledProbeRule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_global_dashboard_notices_from_rule(
    items: &[probe_probe_estimate::ScheduledProbeRule],
) -> Vec<GlobalDashboardNotice> {
    let mut out: Vec<GlobalDashboardNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_global_dashboard_notice(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_global_dashboard_notice(item: &GlobalDashboardNotice) -> String {
    probe_probe_estimate::classify_adaptive_probe_estimate(item.net_volume())
}
