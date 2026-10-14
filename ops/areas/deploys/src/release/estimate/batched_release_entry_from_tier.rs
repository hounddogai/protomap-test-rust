use super::*;

use std::collections::BTreeMap;

use crate::canary::proposal as canary_proposal;

pub fn summarize_adaptive_canary_tiers_for_entry(
    items: &[canary_proposal::AdaptiveCanaryTier],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_batched_release_entries_from_tier(
    items: &[canary_proposal::AdaptiveCanaryTier],
) -> Vec<BatchedReleaseEntry> {
    let mut out: Vec<BatchedReleaseEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_release_entry(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_batched_release_entry(item: &BatchedReleaseEntry) -> String {
    canary_proposal::strict_canary_proposal_bucket(item.measured_value())
}
