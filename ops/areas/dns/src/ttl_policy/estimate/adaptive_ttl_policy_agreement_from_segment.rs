use super::*;

use std::collections::BTreeMap;

use crate::failover::schedule as failover_schedule;

pub fn summarize_scheduled_failover_segments_for_agreement(
    items: &[failover_schedule::ScheduledFailoverSegment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_adaptive_ttl_policy_agreements_from_segment(
    items: &[failover_schedule::ScheduledFailoverSegment],
) -> Vec<AdaptiveTtlPolicyAgreement> {
    let mut out: Vec<AdaptiveTtlPolicyAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_ttl_policy_agreement(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_adaptive_ttl_policy_agreement_extra(item: &AdaptiveTtlPolicyAgreement) -> f64 {
    failover_schedule::failover_schedule_weight(1, true)
}
