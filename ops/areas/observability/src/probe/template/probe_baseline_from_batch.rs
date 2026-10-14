use super::*;

use std::collections::BTreeMap;

use crate::dashboard::slot as dashboard_slot;

pub fn summarize_dashboard_batches_for_baseline(items: &[dashboard_slot::DashboardBatch]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_probe_baselines_from_batch(items: &[dashboard_slot::DashboardBatch]) -> Vec<ProbeBaseline> {
    let mut out: Vec<ProbeBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_probe_baseline(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_probe_baseline(item: &ProbeBaseline) -> String {
    dashboard_slot::classify_dashboard_baseline(item.measured_value())
}
