use super::*;

use std::collections::BTreeMap;

use crate::rule::review as rule_review;

pub fn summarize_regional_rule_reviews_for_metric(items: &[rule_review::RegionalRuleReview]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_egress_metrics_from_review(items: &[rule_review::RegionalRuleReview]) -> Vec<EgressMetric> {
    let mut out: Vec<EgressMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_egress_metric(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_egress_metric(item: &EgressMetric) -> String {
    rule_review::classify_rule_preference(item.net_exposure())
}
