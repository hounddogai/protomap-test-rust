use super::*;

use std::collections::BTreeMap;

use crate::dashboard::dashboard_ledger as dashboard_dashboard_ledger;

pub fn summarize_dashboard_records_for_baseline(
    items: &[dashboard_dashboard_ledger::DashboardRecord],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_alert_baselines_from_record(items: &[dashboard_dashboard_ledger::DashboardRecord]) -> Vec<AlertBaseline> {
    let mut out: Vec<AlertBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_alert_baseline(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_alert_baseline(item: &AlertBaseline) -> String {
    dashboard_dashboard_ledger::normalize_dashboard_record_code(&item.code)
}
