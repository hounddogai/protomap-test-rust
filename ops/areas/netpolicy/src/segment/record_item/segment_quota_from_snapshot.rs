use super::*;

use std::collections::BTreeMap;

use crate::allowlist::allowlist_snapshot as allowlist_allowlist_snapshot;

pub fn summarize_batched_allowlist_snapshots_for_quota(
    items: &[allowlist_allowlist_snapshot::BatchedAllowlistSnapshot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_segment_quotas_from_snapshot(
    items: &[allowlist_allowlist_snapshot::BatchedAllowlistSnapshot],
) -> Vec<SegmentQuota> {
    let mut out: Vec<SegmentQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_segment_quota(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_segment_quota(item: &SegmentQuota) -> String {
    allowlist_allowlist_snapshot::classify_batched_allowlist_snapshot(item.measured_value())
}
