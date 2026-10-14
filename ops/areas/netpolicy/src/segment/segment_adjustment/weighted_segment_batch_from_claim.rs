use super::*;

use std::collections::BTreeMap;

use crate::allowlist::notice as allowlist_notice;

pub fn summarize_tiered_allowlist_claims_for_batch(
    items: &[allowlist_notice::TieredAllowlistClaim],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_weighted_segment_batches_from_claim(
    items: &[allowlist_notice::TieredAllowlistClaim],
) -> Vec<WeightedSegmentBatch> {
    let mut out: Vec<WeightedSegmentBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_segment_batch(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_weighted_segment_batch(item: &WeightedSegmentBatch) -> String {
    allowlist_notice::tiered_allowlist_claim_bucket(item.weighted_load())
}
