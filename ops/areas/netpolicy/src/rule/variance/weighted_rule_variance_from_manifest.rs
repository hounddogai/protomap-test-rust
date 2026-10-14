use super::*;

use std::collections::BTreeMap;

use crate::rule::assignment as rule_assignment;

pub fn summarize_rule_manifests_for_variance(items: &[rule_assignment::RuleManifest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_weighted_rule_variances_from_manifest(
    items: &[rule_assignment::RuleManifest],
) -> Vec<WeightedRuleVariance> {
    let mut out: Vec<WeightedRuleVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_rule_variance(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_weighted_rule_variance(item: &WeightedRuleVariance) -> String {
    rule_assignment::lenient_rule_draft_bucket(item.adjusted_exposure())
}
