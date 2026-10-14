use super::*;

use std::collections::BTreeMap;

use crate::limit::limit_override as limit_limit_override;

pub fn summarize_tiered_limit_allocations_for_score(
    items: &[limit_limit_override::TieredLimitAllocation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_limit_scores_from_allocation(items: &[limit_limit_override::TieredLimitAllocation]) -> Vec<LimitScore> {
    let mut out: Vec<LimitScore> = Vec::new();
    for item in items.iter() {
        let derived = new_limit_score(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_limit_score(item: &LimitScore) -> String {
    limit_limit_override::limit_batch_bucket(item.effective_exposure())
}
