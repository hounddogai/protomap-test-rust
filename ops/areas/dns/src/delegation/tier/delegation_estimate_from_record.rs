use super::*;

use std::collections::BTreeMap;

use crate::health_check::assignment as health_check_assignment;

pub fn summarize_health_check_records_for_estimate(
    items: &[health_check_assignment::HealthCheckRecord],
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

pub fn derive_delegation_estimates_from_record(
    items: &[health_check_assignment::HealthCheckRecord],
) -> Vec<DelegationEstimate> {
    let mut out: Vec<DelegationEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_delegation_estimate(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_delegation_estimate(item: &DelegationEstimate) -> String {
    health_check_assignment::classify_health_check_assignment(item.adjusted_value())
}
