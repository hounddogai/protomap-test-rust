use super::*;

use std::collections::BTreeMap;

use crate::rule::review as rule_review;

pub fn summarize_regional_rule_reviews_for_allocation(
    items: &[rule_review::RegionalRuleReview],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_rule_allocations_from_review(items: &[rule_review::RegionalRuleReview]) -> Vec<RuleAllocation> {
    let mut out: Vec<RuleAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_rule_allocation(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_rule_allocation(item: &RuleAllocation) -> String {
    rule_review::classify_rule_preference(item.adjusted_load())
}
