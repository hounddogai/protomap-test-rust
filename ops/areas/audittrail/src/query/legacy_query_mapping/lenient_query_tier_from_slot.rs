use super::*;

use std::collections::BTreeMap;

use crate::query::adaptive_query_manifest as query_adaptive_query_manifest;

pub fn summarize_batched_query_slots_for_tier(
    items: &[query_adaptive_query_manifest::BatchedQuerySlot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_lenient_query_tiers_from_slot(
    items: &[query_adaptive_query_manifest::BatchedQuerySlot],
) -> Vec<LenientQueryTier> {
    let mut out: Vec<LenientQueryTier> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_query_tier(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_lenient_query_tier(item: &LenientQueryTier) -> String {
    query_adaptive_query_manifest::batched_query_slot_bucket(item.adjusted_value())
}
