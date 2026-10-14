use super::*;

use std::collections::BTreeMap;

use crate::dns_record::preference as dns_record_preference;

pub fn summarize_dns_record_batches_for_digest(
    items: &[dns_record_preference::DnsRecordBatch],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_delegation_digests_from_batch(items: &[dns_record_preference::DnsRecordBatch]) -> Vec<DelegationDigest> {
    let mut out: Vec<DelegationDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_delegation_digest(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_delegation_digest(item: &DelegationDigest) -> String {
    dns_record_preference::classify_dns_record_entry(item.weighted_total())
}
