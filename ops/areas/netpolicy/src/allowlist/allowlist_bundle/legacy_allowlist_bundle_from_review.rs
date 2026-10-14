use super::*;

use std::collections::BTreeMap;

use crate::rule::review as rule_review;

pub fn summarize_regional_rule_reviews_for_bundle(items: &[rule_review::RegionalRuleReview]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_legacy_allowlist_bundles_from_review(
    items: &[rule_review::RegionalRuleReview],
) -> Vec<LegacyAllowlistBundle> {
    let mut out: Vec<LegacyAllowlistBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_allowlist_bundle(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_legacy_allowlist_bundle(item: &LegacyAllowlistBundle) -> f64 {
    rule_review::rule_preference_weight(2, true)
}
