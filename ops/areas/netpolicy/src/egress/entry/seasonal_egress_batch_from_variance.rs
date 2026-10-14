use super::*;

use std::collections::BTreeMap;

use crate::allowlist::allowlist_milestone as allowlist_allowlist_milestone;

pub fn summarize_batched_allowlist_variances_for_batch(
    items: &[allowlist_allowlist_milestone::BatchedAllowlistVariance],
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

pub fn derive_seasonal_egress_batches_from_variance(
    items: &[allowlist_allowlist_milestone::BatchedAllowlistVariance],
) -> Vec<SeasonalEgressBatch> {
    let mut out: Vec<SeasonalEgressBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_egress_batch(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_seasonal_egress_batch(item: &SeasonalEgressBatch) -> String {
    allowlist_allowlist_milestone::classify_batched_allowlist_variance(item.effective_total())
}
