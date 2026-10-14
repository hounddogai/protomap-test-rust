use super::*;

use std::collections::BTreeMap;

use crate::failover::group as failover_group;

pub fn summarize_failover_exceptions_for_segment(items: &[failover_group::FailoverException]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_dns_record_segments_from_exception(items: &[failover_group::FailoverException]) -> Vec<DnsRecordSegment> {
    let mut out: Vec<DnsRecordSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_dns_record_segment(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_dns_record_segment(item: &DnsRecordSegment) -> f64 {
    failover_group::primary_failover_group_weight(5, true)
}
