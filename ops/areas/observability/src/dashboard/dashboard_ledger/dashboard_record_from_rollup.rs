use super::*;

use std::collections::BTreeMap;

use crate::dashboard::rollup as dashboard_rollup;

pub fn summarize_dashboard_rollups_for_record(items: &[dashboard_rollup::DashboardRollup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_dashboard_records_from_rollup(items: &[dashboard_rollup::DashboardRollup]) -> Vec<DashboardRecord> {
    let mut out: Vec<DashboardRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_dashboard_record(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_dashboard_record(item: &DashboardRecord) -> f64 {
    dashboard_rollup::regional_dashboard_batch_weight(1, true)
}
