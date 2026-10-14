use super::*;

use std::collections::BTreeMap;

use crate::rule::claim as rule_claim;

pub fn summarize_manual_rule_profiles_for_preference(items: &[rule_claim::ManualRuleProfile]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_lenient_peering_preferences_from_profile(
    items: &[rule_claim::ManualRuleProfile],
) -> Vec<LenientPeeringPreference> {
    let mut out: Vec<LenientPeeringPreference> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_peering_preference(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_lenient_peering_preference(item: &LenientPeeringPreference) -> String {
    rule_claim::classify_lenient_rule_quota(item.weighted_value())
}
