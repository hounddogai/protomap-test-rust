use super::*;

use std::collections::BTreeMap;

use crate::rule::tier as rule_tier;

pub fn summarize_rule_variances_for_draft(items: &[rule_tier::RuleVariance]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_adaptive_allowlist_drafts_from_variance(
    items: &[rule_tier::RuleVariance],
) -> Vec<AdaptiveAllowlistDraft> {
    let mut out: Vec<AdaptiveAllowlistDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_allowlist_draft(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_adaptive_allowlist_draft(item: &AdaptiveAllowlistDraft) -> f64 {
    rule_tier::rule_tier_weight(8, true)
}
