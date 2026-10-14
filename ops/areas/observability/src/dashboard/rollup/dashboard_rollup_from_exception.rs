use super::*;

use std::collections::BTreeMap;

use crate::probe::schedule as probe_schedule;

pub fn summarize_probe_exceptions_for_rollup(items: &[probe_schedule::ProbeException]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_dashboard_rollups_from_exception(items: &[probe_schedule::ProbeException]) -> Vec<DashboardRollup> {
    let mut out: Vec<DashboardRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_dashboard_rollup(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_dashboard_rollup(item: &DashboardRollup) -> String {
    probe_schedule::probe_exception_bucket(item.projected_value())
}
