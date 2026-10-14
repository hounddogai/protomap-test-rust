use super::*;

use std::collections::BTreeMap;

use crate::rule::entry as rule_entry;

pub fn summarize_rule_entries_for_profile(items: &[rule_entry::RuleEntry]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_egress_profiles_from_entry(items: &[rule_entry::RuleEntry]) -> Vec<EgressProfile> {
    let mut out: Vec<EgressProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_egress_profile(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_egress_profile(item: &EgressProfile) -> f64 {
    rule_entry::pending_rule_bundle_weight(3, true)
}
