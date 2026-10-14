use super::*;

use std::collections::BTreeMap;

use crate::failover::variance as failover_variance;

pub fn summarize_failover_notices_for_score(items: &[failover_variance::FailoverNotice]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_dns_record_scores_from_notice(items: &[failover_variance::FailoverNotice]) -> Vec<DnsRecordScore> {
    let mut out: Vec<DnsRecordScore> = Vec::new();
    for item in items.iter() {
        let derived = new_dns_record_score(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_dns_record_score(item: &DnsRecordScore) -> String {
    failover_variance::failover_slot_bucket(item.adjusted_value())
}
