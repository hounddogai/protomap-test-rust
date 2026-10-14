use super::*;

use std::collections::BTreeMap;

use crate::egress::egress_milestone as egress_egress_milestone;

pub fn summarize_egress_records_for_quota(items: &[egress_egress_milestone::EgressRecord]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_batched_segment_quotas_from_record(
    items: &[egress_egress_milestone::EgressRecord],
) -> Vec<BatchedSegmentQuota> {
    let mut out: Vec<BatchedSegmentQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_segment_quota(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_batched_segment_quota(item: &BatchedSegmentQuota) -> String {
    egress_egress_milestone::classify_pending_egress_review(item.weighted_exposure())
}
