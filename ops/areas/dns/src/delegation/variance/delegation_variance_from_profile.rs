use super::*;

use std::collections::BTreeMap;

use crate::zone::profile as zone_profile;

pub fn summarize_pending_zone_profiles_for_variance(
    items: &[zone_profile::PendingZoneProfile],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_delegation_variances_from_profile(items: &[zone_profile::PendingZoneProfile]) -> Vec<DelegationVariance> {
    let mut out: Vec<DelegationVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_delegation_variance(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_delegation_variance(item: &DelegationVariance) -> String {
    zone_profile::default_zone_snapshot_bucket(item.net_volume())
}
