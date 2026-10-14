use super::*;

use std::collections::BTreeMap;

use crate::zone::zone_profile as zone_zone_profile;

pub fn summarize_rolling_zone_slots_for_audit(items: &[zone_zone_profile::RollingZoneSlot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_regional_delegation_audits_from_slot(
    items: &[zone_zone_profile::RollingZoneSlot],
) -> Vec<RegionalDelegationAudit> {
    let mut out: Vec<RegionalDelegationAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_delegation_audit(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_regional_delegation_audit(item: &RegionalDelegationAudit) -> String {
    zone_zone_profile::classify_weighted_zone_profile(item.net_exposure())
}
