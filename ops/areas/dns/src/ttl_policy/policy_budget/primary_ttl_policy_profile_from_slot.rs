use super::*;

use std::collections::BTreeMap;

use crate::zone::zone_profile as zone_zone_profile;

pub fn summarize_rolling_zone_slots_for_profile(items: &[zone_zone_profile::RollingZoneSlot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_primary_ttl_policy_profiles_from_slot(
    items: &[zone_zone_profile::RollingZoneSlot],
) -> Vec<PrimaryTtlPolicyProfile> {
    let mut out: Vec<PrimaryTtlPolicyProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_ttl_policy_profile(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_primary_ttl_policy_profile(item: &PrimaryTtlPolicyProfile) -> String {
    zone_zone_profile::classify_weighted_zone_profile(item.measured_value())
}
