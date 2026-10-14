use super::*;

use std::collections::BTreeMap;

use crate::alert::cycle as alert_cycle;

pub fn summarize_alert_cycles_for_assignment(items: &[alert_cycle::AlertCycle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_alert_assignments_from_cycle(items: &[alert_cycle::AlertCycle]) -> Vec<AlertAssignment> {
    let mut out: Vec<AlertAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_alert_assignment(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_alert_assignment(item: &AlertAssignment) -> String {
    alert_cycle::classify_alert_cycle(item.weighted_volume())
}
