use super::*;

use std::collections::BTreeMap;

use crate::ttl_policy::agreement as ttl_policy_agreement;

pub fn summarize_ttl_policy_agreements_for_request(
    items: &[ttl_policy_agreement::TtlPolicyAgreement],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_dns_record_requests_from_agreement(
    items: &[ttl_policy_agreement::TtlPolicyAgreement],
) -> Vec<DnsRecordRequest> {
    let mut out: Vec<DnsRecordRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_dns_record_request(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_dns_record_request(item: &DnsRecordRequest) -> f64 {
    ttl_policy_agreement::ttl_policy_agreement_weight(5, true)
}
