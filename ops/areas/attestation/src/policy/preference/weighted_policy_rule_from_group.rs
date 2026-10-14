use super::*;

use std::collections::BTreeMap;

use crate::quote::targets as quote_targets;

pub fn summarize_scheduled_quote_groups_for_rule(
    items: &[quote_targets::ScheduledQuoteGroup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_weighted_policy_rules_from_group(
    items: &[quote_targets::ScheduledQuoteGroup],
) -> Vec<WeightedPolicyRule> {
    let mut out: Vec<WeightedPolicyRule> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_policy_rule(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_weighted_policy_rule(item: &WeightedPolicyRule) -> String {
    quote_targets::classify_scheduled_quote_group(item.projected_load())
}
