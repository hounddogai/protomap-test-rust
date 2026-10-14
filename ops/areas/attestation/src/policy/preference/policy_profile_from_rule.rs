use super::*;

use std::collections::BTreeMap;

use crate::policy::rule as policy_rule;

pub fn summarize_policy_rules_for_profile(items: &[policy_rule::PolicyRule]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_policy_profiles_from_rule(items: &[policy_rule::PolicyRule]) -> Vec<PolicyProfile> {
    let mut out: Vec<PolicyProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_policy_profile(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_policy_profile(item: &PolicyProfile) -> String {
    policy_rule::policy_rule_bucket(item.weighted_volume())
}
