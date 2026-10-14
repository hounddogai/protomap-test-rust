use super::*;

use std::collections::BTreeMap;

use crate::failover::segment as failover_segment;

pub fn summarize_weighted_failover_segments_for_threshold(
    items: &[failover_segment::WeightedFailoverSegment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_pending_ttl_policy_thresholds_from_segment(
    items: &[failover_segment::WeightedFailoverSegment],
) -> Vec<PendingTtlPolicyThreshold> {
    let mut out: Vec<PendingTtlPolicyThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_ttl_policy_threshold(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_pending_ttl_policy_threshold(item: &PendingTtlPolicyThreshold) -> String {
    failover_segment::weighted_failover_segment_bucket(item.effective_value())
}
