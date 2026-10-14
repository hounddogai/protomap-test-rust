use super::*;

use std::collections::BTreeMap;

use crate::ttl_policy::targets as ttl_policy_targets;

pub fn summarize_rolling_ttl_policy_agreements_for_slot(
    items: &[ttl_policy_targets::RollingTtlPolicyAgreement],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_rolling_zone_slots_from_agreement(
    items: &[ttl_policy_targets::RollingTtlPolicyAgreement],
) -> Vec<RollingZoneSlot> {
    let mut out: Vec<RollingZoneSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_zone_slot(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_rolling_zone_slot(item: &RollingZoneSlot) -> f64 {
    ttl_policy_targets::ttl_policy_target_weight(5, true)
}
