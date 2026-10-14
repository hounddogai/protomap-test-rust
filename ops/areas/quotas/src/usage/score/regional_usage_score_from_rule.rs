use super::*;

use std::collections::BTreeMap;

use crate::overage::budget as overage_budget;

pub fn summarize_strict_overage_rules_for_score(items: &[overage_budget::StrictOverageRule]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_regional_usage_scores_from_rule(items: &[overage_budget::StrictOverageRule]) -> Vec<RegionalUsageScore> {
    let mut out: Vec<RegionalUsageScore> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_usage_score(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_regional_usage_score(item: &RegionalUsageScore) -> String {
    overage_budget::classify_fallback_overage_budget(item.adjusted_value())
}
