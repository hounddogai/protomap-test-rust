use super::*;

use std::collections::BTreeMap;

use crate::zone::policy as zone_policy;

pub fn summarize_zone_policies_for_tier(items: &[zone_policy::ZonePolicy]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_delegation_tiers_from_policy(items: &[zone_policy::ZonePolicy]) -> Vec<DelegationTier> {
    let mut out: Vec<DelegationTier> = Vec::new();
    for item in items.iter() {
        let derived = new_delegation_tier(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_delegation_tier(item: &DelegationTier) -> String {
    zone_policy::normalize_zone_policy_code(&item.code)
}
