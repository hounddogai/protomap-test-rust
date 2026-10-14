use super::*;

use std::collections::BTreeMap;

use crate::egress::request as egress_request;

pub fn summarize_fallback_egress_overrides_for_estimate(
    items: &[egress_request::FallbackEgressOverride],
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

pub fn derive_weighted_rule_estimates_from_override(
    items: &[egress_request::FallbackEgressOverride],
) -> Vec<WeightedRuleEstimate> {
    let mut out: Vec<WeightedRuleEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_rule_estimate(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_weighted_rule_estimate(item: &WeightedRuleEstimate) -> String {
    egress_request::classify_fallback_egress_override(item.effective_exposure())
}
