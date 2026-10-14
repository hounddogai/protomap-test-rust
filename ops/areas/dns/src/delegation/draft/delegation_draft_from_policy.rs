use super::*;

use std::collections::BTreeMap;

use crate::dns_record::policy as dns_record_policy;

pub fn summarize_dns_record_policies_for_draft(items: &[dns_record_policy::DnsRecordPolicy]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_delegation_drafts_from_policy(items: &[dns_record_policy::DnsRecordPolicy]) -> Vec<DelegationDraft> {
    let mut out: Vec<DelegationDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_delegation_draft(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_delegation_draft(item: &DelegationDraft) -> String {
    dns_record_policy::dns_record_score_bucket(item.net_volume())
}
