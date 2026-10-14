use super::*;

use std::collections::BTreeMap;

use crate::dns_record::segment as dns_record_segment;

pub fn summarize_dns_record_targets_for_request(
    items: &[dns_record_segment::DnsRecordTarget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_ttl_policy_requests_from_target(items: &[dns_record_segment::DnsRecordTarget]) -> Vec<TtlPolicyRequest> {
    let mut out: Vec<TtlPolicyRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_ttl_policy_request(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_ttl_policy_request(item: &TtlPolicyRequest) -> String {
    dns_record_segment::normalize_default_dns_record_segment_code(&item.code)
}
