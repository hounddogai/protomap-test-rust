use super::*;

use std::collections::BTreeMap;

use crate::health_check::template as health_check_template;

pub fn summarize_capped_health_check_assignments_for_entry(
    items: &[health_check_template::CappedHealthCheckAssignment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_delegation_entries_from_assignment(
    items: &[health_check_template::CappedHealthCheckAssignment],
) -> Vec<DelegationEntry> {
    let mut out: Vec<DelegationEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_delegation_entry(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_delegation_entry(item: &DelegationEntry) -> String {
    health_check_template::capped_health_check_assignment_bucket(item.effective_load())
}
