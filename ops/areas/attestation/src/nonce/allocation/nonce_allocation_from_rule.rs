use super::*;

use std::collections::BTreeMap;

use crate::measurement::measurement_notice as measurement_measurement_notice;

pub fn summarize_measurement_rules_for_allocation(
    items: &[measurement_measurement_notice::MeasurementRule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_nonce_allocations_from_rule(
    items: &[measurement_measurement_notice::MeasurementRule],
) -> Vec<NonceAllocation> {
    let mut out: Vec<NonceAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_nonce_allocation(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_nonce_allocation(item: &NonceAllocation) -> String {
    measurement_measurement_notice::classify_legacy_measurement_notice(item.measured_value())
}
