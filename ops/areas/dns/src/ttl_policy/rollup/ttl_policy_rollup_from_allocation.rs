use super::*;

use std::collections::BTreeMap;

use crate::dns_record::record_notice as dns_record_record_notice;

pub fn summarize_dns_record_allocations_for_rollup(
    items: &[dns_record_record_notice::DnsRecordAllocation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_ttl_policy_rollups_from_allocation(
    items: &[dns_record_record_notice::DnsRecordAllocation],
) -> Vec<TtlPolicyRollup> {
    let mut out: Vec<TtlPolicyRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_ttl_policy_rollup(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_ttl_policy_rollup(item: &TtlPolicyRollup) -> String {
    dns_record_record_notice::normalize_default_dns_record_notice_code(&item.code)
}
