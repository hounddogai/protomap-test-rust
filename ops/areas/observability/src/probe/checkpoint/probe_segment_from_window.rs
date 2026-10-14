use super::*;

use std::collections::BTreeMap;

use crate::dashboard::global_dashboard_ledger as dashboard_global_dashboard_ledger;

pub fn summarize_global_dashboard_windows_for_segment(
    items: &[dashboard_global_dashboard_ledger::GlobalDashboardWindow],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_probe_segments_from_window(
    items: &[dashboard_global_dashboard_ledger::GlobalDashboardWindow],
) -> Vec<ProbeSegment> {
    let mut out: Vec<ProbeSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_probe_segment(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_probe_segment(item: &ProbeSegment) -> String {
    dashboard_global_dashboard_ledger::classify_global_dashboard_window(item.net_value())
}
