use super::*;

use std::collections::BTreeMap;

use crate::rule::notice as rule_notice;

pub fn summarize_rule_estimates_for_score(items: &[rule_notice::RuleEstimate]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_peering_scores_from_estimate(items: &[rule_notice::RuleEstimate]) -> Vec<PeeringScore> {
    let mut out: Vec<PeeringScore> = Vec::new();
    for item in items.iter() {
        let derived = new_peering_score(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_peering_score(item: &PeeringScore) -> f64 {
    rule_notice::rule_estimate_weight(5, true)
}
