use super::*;

use std::collections::BTreeMap;

use crate::ttl_policy::template as ttl_policy_template;

pub fn summarize_ttl_policy_requests_for_override(
    items: &[ttl_policy_template::TtlPolicyRequest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_dns_record_overrides_from_request(
    items: &[ttl_policy_template::TtlPolicyRequest],
) -> Vec<DnsRecordOverride> {
    let mut out: Vec<DnsRecordOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_dns_record_override(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_dns_record_override(item: &DnsRecordOverride) -> String {
    ttl_policy_template::classify_ttl_policy_milestone(item.measured_value())
}
