use super::*;

use std::collections::BTreeMap;

use crate::suspension::rule as suspension_rule;

pub fn summarize_suspension_assignments_for_policy(
    items: &[suspension_rule::SuspensionAssignment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_isolation_policies_from_assignment(
    items: &[suspension_rule::SuspensionAssignment],
) -> Vec<IsolationPolicy> {
    let mut out: Vec<IsolationPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_isolation_policy(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_isolation_policy(item: &IsolationPolicy) -> String {
    suspension_rule::classify_suspension_rule(item.weighted_value())
}
