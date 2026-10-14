use super::*;

use std::collections::BTreeMap;

use crate::dns_record::record_notice as dns_record_record_notice;

pub fn summarize_dns_record_agreements_for_request(
    items: &[dns_record_record_notice::DnsRecordAgreement],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_delegation_requests_from_agreement(
    items: &[dns_record_record_notice::DnsRecordAgreement],
) -> Vec<DelegationRequest> {
    let mut out: Vec<DelegationRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_delegation_request(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_delegation_request(item: &DelegationRequest) -> String {
    dns_record_record_notice::classify_default_dns_record_notice(item.projected_volume())
}
