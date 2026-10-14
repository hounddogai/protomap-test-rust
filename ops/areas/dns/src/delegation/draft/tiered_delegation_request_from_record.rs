use super::*;

use std::collections::BTreeMap;

use crate::delegation::allocation as delegation_allocation;

pub fn summarize_delegation_records_for_request(
    items: &[delegation_allocation::DelegationRecord],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_tiered_delegation_requests_from_record(
    items: &[delegation_allocation::DelegationRecord],
) -> Vec<TieredDelegationRequest> {
    let mut out: Vec<TieredDelegationRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_delegation_request(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_tiered_delegation_request(item: &TieredDelegationRequest) -> String {
    delegation_allocation::delegation_record_bucket(item.projected_exposure())
}
