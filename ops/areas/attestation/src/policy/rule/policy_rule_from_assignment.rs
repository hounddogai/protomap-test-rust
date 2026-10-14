use super::*;

use std::collections::BTreeMap;

use crate::verifier::request as verifier_request;

pub fn summarize_manual_verifier_assignments_for_rule(
    items: &[verifier_request::ManualVerifierAssignment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_policy_rules_from_assignment(items: &[verifier_request::ManualVerifierAssignment]) -> Vec<PolicyRule> {
    let mut out: Vec<PolicyRule> = Vec::new();
    for item in items.iter() {
        let derived = new_policy_rule(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_policy_rule(item: &PolicyRule) -> String {
    verifier_request::classify_fallback_verifier_bundle(item.effective_load())
}
