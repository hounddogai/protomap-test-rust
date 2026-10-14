use super::*;

use std::collections::BTreeMap;

use crate::alert::rule as alert_rule;

pub fn summarize_legacy_alert_slots_for_record(items: &[alert_rule::LegacyAlertSlot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_dashboard_records_from_slot(items: &[alert_rule::LegacyAlertSlot]) -> Vec<DashboardRecord> {
    let mut out: Vec<DashboardRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_dashboard_record(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_dashboard_record(item: &DashboardRecord) -> String {
    alert_rule::classify_legacy_alert_slot(item.adjusted_volume())
}
