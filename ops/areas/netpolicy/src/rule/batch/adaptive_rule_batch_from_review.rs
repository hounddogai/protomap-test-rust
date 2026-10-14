use super::*;

use std::collections::BTreeMap;

use crate::rule::rule_review as rule_rule_review;

pub fn summarize_manual_rule_reviews_for_batch(items: &[rule_rule_review::ManualRuleReview]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_adaptive_rule_batches_from_review(
    items: &[rule_rule_review::ManualRuleReview],
) -> Vec<AdaptiveRuleBatch> {
    let mut out: Vec<AdaptiveRuleBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_rule_batch(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_adaptive_rule_batch(item: &AdaptiveRuleBatch) -> String {
    rule_rule_review::rule_claim_bucket(item.projected_volume())
}
