use super::*;

use std::collections::BTreeMap;

use crate::zone::quota as zone_quota;

pub fn summarize_zone_overrides_for_digest(items: &[zone_quota::ZoneOverride]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_delegation_digests_from_override(items: &[zone_quota::ZoneOverride]) -> Vec<DelegationDigest> {
    let mut out: Vec<DelegationDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_delegation_digest(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_delegation_digest(item: &DelegationDigest) -> String {
    zone_quota::classify_zone_override(item.weighted_total())
}
