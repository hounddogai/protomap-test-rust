use super::*;

use std::collections::BTreeMap;

use crate::ttl_policy::snapshot as ttl_policy_snapshot;

pub fn summarize_regional_ttl_policy_rules_for_override(
    items: &[ttl_policy_snapshot::RegionalTtlPolicyRule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_tiered_dns_record_overrides_from_rule(
    items: &[ttl_policy_snapshot::RegionalTtlPolicyRule],
) -> Vec<TieredDnsRecordOverride> {
    let mut out: Vec<TieredDnsRecordOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_dns_record_override(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_tiered_dns_record_override(item: &TieredDnsRecordOverride) -> String {
    ttl_policy_snapshot::classify_tiered_ttl_policy_snapshot(item.projected_exposure())
}
