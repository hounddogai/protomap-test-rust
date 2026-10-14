use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::baseline as budget_burn_baseline;

pub fn summarize_batched_budget_burn_reviews_for_score(
    items: &[budget_burn_baseline::BatchedBudgetBurnReview],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_rolling_silence_scores_from_review(
    items: &[budget_burn_baseline::BatchedBudgetBurnReview],
) -> Vec<RollingSilenceScore> {
    let mut out: Vec<RollingSilenceScore> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_silence_score(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_rolling_silence_score(item: &RollingSilenceScore) -> f64 {
    budget_burn_baseline::batched_budget_burn_batch_weight(1, true)
}
