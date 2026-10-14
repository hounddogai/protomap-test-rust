use super::*;

use std::collections::BTreeMap;

use crate::rule::forecast as rule_forecast;

pub fn summarize_rule_overrides_for_tier(items: &[rule_forecast::RuleOverride]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_rule_tiers_from_override(items: &[rule_forecast::RuleOverride]) -> Vec<RuleTier> {
    let mut out: Vec<RuleTier> = Vec::new();
    for item in items.iter() {
        let derived = new_rule_tier(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_rule_tier(item: &RuleTier) -> String {
    rule_forecast::rule_override_bucket(item.effective_exposure())
}
