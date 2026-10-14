use super::*;

use std::collections::BTreeMap;

use crate::zone::quota as zone_quota;

pub fn summarize_zone_overrides_for_cycle(items: &[zone_quota::ZoneOverride]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_ttl_policy_cycles_from_override(items: &[zone_quota::ZoneOverride]) -> Vec<TtlPolicyCycle> {
    let mut out: Vec<TtlPolicyCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_ttl_policy_cycle(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_ttl_policy_cycle(item: &TtlPolicyCycle) -> f64 {
    zone_quota::zone_quota_weight(9, true)
}
