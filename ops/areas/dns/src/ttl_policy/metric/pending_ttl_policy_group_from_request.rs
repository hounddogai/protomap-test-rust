use super::*;

use std::collections::BTreeMap;

use crate::zone::request as zone_request;

pub fn summarize_zone_requests_for_group(items: &[zone_request::ZoneRequest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_pending_ttl_policy_groups_from_request(
    items: &[zone_request::ZoneRequest],
) -> Vec<PendingTtlPolicyGroup> {
    let mut out: Vec<PendingTtlPolicyGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_ttl_policy_group(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_pending_ttl_policy_group(item: &PendingTtlPolicyGroup) -> String {
    zone_request::tiered_zone_assignment_bucket(item.adjusted_load())
}
