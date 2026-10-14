use super::*;

use std::collections::BTreeMap;

use crate::probe::probe_request as probe_probe_request;

pub fn summarize_default_probe_schedules_for_template(
    items: &[probe_probe_request::DefaultProbeSchedule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_primary_dashboard_templates_from_schedule(
    items: &[probe_probe_request::DefaultProbeSchedule],
) -> Vec<PrimaryDashboardTemplate> {
    let mut out: Vec<PrimaryDashboardTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_dashboard_template(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_primary_dashboard_template(item: &PrimaryDashboardTemplate) -> String {
    probe_probe_request::classify_probe_variance(item.adjusted_value())
}
