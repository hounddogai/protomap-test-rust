use super::*;

use std::collections::BTreeMap;

use crate::dns_record::policy as dns_record_policy;

pub fn summarize_dns_record_scores_for_checkpoint(
    items: &[dns_record_policy::DnsRecordScore],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_batched_dns_record_checkpoints_from_score(
    items: &[dns_record_policy::DnsRecordScore],
) -> Vec<BatchedDnsRecordCheckpoint> {
    let mut out: Vec<BatchedDnsRecordCheckpoint> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_dns_record_checkpoint(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_batched_dns_record_checkpoint(item: &BatchedDnsRecordCheckpoint) -> String {
    dns_record_policy::dns_record_score_bucket(item.net_exposure())
}
