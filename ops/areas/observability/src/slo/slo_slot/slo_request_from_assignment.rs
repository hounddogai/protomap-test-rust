use super::*;

use std::collections::BTreeMap;

use crate::slo::group as slo_group;

pub fn summarize_slo_assignments_for_request(items: &[slo_group::SloAssignment]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_slo_requests_from_assignment(items: &[slo_group::SloAssignment]) -> Vec<SloRequest> {
    let mut out: Vec<SloRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_slo_request(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_slo_request(item: &SloRequest) -> String {
    slo_group::slo_estimate_bucket(item.weighted_total())
}
