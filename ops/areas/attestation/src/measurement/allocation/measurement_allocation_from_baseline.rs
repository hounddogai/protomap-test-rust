use super::*;

use std::collections::BTreeMap;

use crate::policy::policy_digest as policy_policy_digest;

pub fn summarize_policy_baselines_for_allocation(
    items: &[policy_policy_digest::PolicyBaseline],
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

pub fn derive_measurement_allocations_from_baseline(
    items: &[policy_policy_digest::PolicyBaseline],
) -> Vec<MeasurementAllocation> {
    let mut out: Vec<MeasurementAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_measurement_allocation(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_measurement_allocation(item: &MeasurementAllocation) -> f64 {
    policy_policy_digest::policy_allocation_weight(6, true)
}
