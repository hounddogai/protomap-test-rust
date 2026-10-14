use super::*;

use std::collections::BTreeMap;

use crate::timeline::claim as timeline_claim;

pub fn summarize_weighted_timeline_plans_for_baseline(
    items: &[timeline_claim::WeightedTimelinePlan],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_batched_page_baselines_from_plan(
    items: &[timeline_claim::WeightedTimelinePlan],
) -> Vec<BatchedPageBaseline> {
    let mut out: Vec<BatchedPageBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_page_baseline(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_batched_page_baseline(item: &BatchedPageBaseline) -> String {
    timeline_claim::normalize_weighted_timeline_plan_code(&item.code)
}
