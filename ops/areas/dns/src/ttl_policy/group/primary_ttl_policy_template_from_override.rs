use super::*;

use std::collections::BTreeMap;

use crate::zone::zone_quota as zone_zone_quota;

pub fn summarize_scheduled_zone_overrides_for_template(
    items: &[zone_zone_quota::ScheduledZoneOverride],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_primary_ttl_policy_templates_from_override(
    items: &[zone_zone_quota::ScheduledZoneOverride],
) -> Vec<PrimaryTtlPolicyTemplate> {
    let mut out: Vec<PrimaryTtlPolicyTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_ttl_policy_template(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_primary_ttl_policy_template(item: &PrimaryTtlPolicyTemplate) -> String {
    zone_zone_quota::normalize_scheduled_zone_override_code(&item.code)
}
