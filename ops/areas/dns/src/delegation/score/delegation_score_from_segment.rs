use super::*;

use std::collections::BTreeMap;

use crate::dns_record::item as dns_record_item;

pub fn summarize_dns_record_segments_for_score(items: &[dns_record_item::DnsRecordSegment]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_delegation_scores_from_segment(items: &[dns_record_item::DnsRecordSegment]) -> Vec<DelegationScore> {
    let mut out: Vec<DelegationScore> = Vec::new();
    for item in items.iter() {
        let derived = new_delegation_score(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_delegation_score(item: &DelegationScore) -> String {
    dns_record_item::classify_dns_record_segment(item.projected_volume())
}
