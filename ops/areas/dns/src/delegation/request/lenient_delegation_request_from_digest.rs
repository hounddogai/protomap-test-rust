use super::*;

use std::collections::BTreeMap;

use crate::zone::zone_digest as zone_zone_digest;

pub fn summarize_global_zone_digests_for_request(
    items: &[zone_zone_digest::GlobalZoneDigest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_lenient_delegation_requests_from_digest(
    items: &[zone_zone_digest::GlobalZoneDigest],
) -> Vec<LenientDelegationRequest> {
    let mut out: Vec<LenientDelegationRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_delegation_request(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn derive_lenient_delegation_request(item: &LenientDelegationRequest) -> String {
    zone_zone_digest::classify_global_zone_digest(item.net_volume())
}
