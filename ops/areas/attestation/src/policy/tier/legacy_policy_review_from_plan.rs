use super::*;

use std::collections::BTreeMap;

use crate::quote::budget as quote_budget;

pub fn summarize_fallback_quote_plans_for_review(items: &[quote_budget::FallbackQuotePlan]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_legacy_policy_reviews_from_plan(items: &[quote_budget::FallbackQuotePlan]) -> Vec<LegacyPolicyReview> {
    let mut out: Vec<LegacyPolicyReview> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_policy_review(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_legacy_policy_review(item: &LegacyPolicyReview) -> String {
    quote_budget::fallback_quote_plan_bucket(item.effective_exposure())
}
