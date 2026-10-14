use super::*;

use std::collections::BTreeMap;

use crate::policy::summary as policy_summary;

pub fn summarize_policy_estimates_for_rollup(items: &[policy_summary::PolicyEstimate]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_measurement_rollups_from_estimate(items: &[policy_summary::PolicyEstimate]) -> Vec<MeasurementRollup> {
    let mut out: Vec<MeasurementRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_measurement_rollup(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_measurement_rollup(item: &MeasurementRollup) -> f64 {
    policy_summary::policy_summary_weight(8, true)
}
