use super::*;

use std::collections::BTreeMap;

use crate::failover::budget as failover_budget;

pub fn summarize_batched_failover_reviews_for_tier(
    items: &[failover_budget::BatchedFailoverReview],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_primary_dns_record_tiers_from_review(
    items: &[failover_budget::BatchedFailoverReview],
) -> Vec<PrimaryDnsRecordTier> {
    let mut out: Vec<PrimaryDnsRecordTier> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_dns_record_tier(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_primary_dns_record_tier(item: &PrimaryDnsRecordTier) -> f64 {
    failover_budget::batched_failover_review_weight(1, true)
}
