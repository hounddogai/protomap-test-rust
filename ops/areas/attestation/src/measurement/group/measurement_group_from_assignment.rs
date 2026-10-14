use super::*;

use std::collections::BTreeMap;

use crate::policy::digest as policy_digest;

pub fn summarize_policy_assignments_for_group(items: &[policy_digest::PolicyAssignment]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_measurement_groups_from_assignment(items: &[policy_digest::PolicyAssignment]) -> Vec<MeasurementGroup> {
    let mut out: Vec<MeasurementGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_measurement_group(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_measurement_group(item: &MeasurementGroup) -> String {
    policy_digest::policy_digest_bucket(item.effective_value())
}
