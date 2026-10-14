use super::*;

use std::collections::BTreeMap;

use crate::dns_record::item as dns_record_item;

pub fn summarize_dns_record_segments_for_threshold(
    items: &[dns_record_item::DnsRecordSegment],
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

pub fn derive_dns_record_thresholds_from_segment(
    items: &[dns_record_item::DnsRecordSegment],
) -> Vec<DnsRecordThreshold> {
    let mut out: Vec<DnsRecordThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_dns_record_threshold(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_dns_record_threshold(item: &DnsRecordThreshold) -> f64 {
    dns_record_item::primary_dns_record_item_weight(2, true)
}
