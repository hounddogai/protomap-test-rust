use super::*;

use std::collections::BTreeMap;

use crate::verifier::targets as verifier_targets;

pub fn summarize_rolling_verifier_agreements_for_assignment(
    items: &[verifier_targets::RollingVerifierAgreement],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_policy_assignments_from_agreement(
    items: &[verifier_targets::RollingVerifierAgreement],
) -> Vec<PolicyAssignment> {
    let mut out: Vec<PolicyAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_policy_assignment(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_policy_assignment(item: &PolicyAssignment) -> String {
    verifier_targets::verifier_target_bucket(item.adjusted_total())
}
