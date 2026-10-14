use super::*;

use std::collections::BTreeMap;

use crate::page::budget as page_budget;

pub fn summarize_batched_page_baselines_for_digest(
    items: &[page_budget::BatchedPageBaseline],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_fallback_action_item_digests_from_baseline(
    items: &[page_budget::BatchedPageBaseline],
) -> Vec<FallbackActionItemDigest> {
    let mut out: Vec<FallbackActionItemDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_action_item_digest(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_fallback_action_item_digest(item: &FallbackActionItemDigest) -> String {
    page_budget::classify_batched_page_baseline(item.net_value())
}
