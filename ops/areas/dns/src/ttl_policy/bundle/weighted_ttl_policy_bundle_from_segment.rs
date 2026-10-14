use super::*;

use std::collections::BTreeMap;

use crate::dns_record::ledger as dns_record_ledger;

pub fn summarize_regional_dns_record_segments_for_bundle(
    items: &[dns_record_ledger::RegionalDnsRecordSegment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_weighted_ttl_policy_bundles_from_segment(
    items: &[dns_record_ledger::RegionalDnsRecordSegment],
) -> Vec<WeightedTtlPolicyBundle> {
    let mut out: Vec<WeightedTtlPolicyBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_ttl_policy_bundle(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_weighted_ttl_policy_bundle(item: &WeightedTtlPolicyBundle) -> String {
    dns_record_ledger::classify_capped_dns_record_entry(item.measured_value())
}
