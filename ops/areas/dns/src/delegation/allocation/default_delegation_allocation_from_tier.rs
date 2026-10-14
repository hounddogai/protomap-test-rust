use super::*;

use std::collections::BTreeMap;

use crate::dns_record::tier as dns_record_tier;

pub fn summarize_primary_dns_record_tiers_for_allocation(
    items: &[dns_record_tier::PrimaryDnsRecordTier],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_default_delegation_allocations_from_tier(
    items: &[dns_record_tier::PrimaryDnsRecordTier],
) -> Vec<DefaultDelegationAllocation> {
    let mut out: Vec<DefaultDelegationAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_default_delegation_allocation(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_default_delegation_allocation(item: &DefaultDelegationAllocation) -> String {
    dns_record_tier::classify_primary_dns_record_tier(item.net_exposure())
}
