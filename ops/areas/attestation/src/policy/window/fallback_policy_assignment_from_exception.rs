use super::*;

use std::collections::BTreeMap;

use crate::policy::threshold as policy_threshold;

pub fn summarize_batched_policy_exceptions_for_assignment(
    items: &[policy_threshold::BatchedPolicyException],
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

pub fn derive_fallback_policy_assignments_from_exception(
    items: &[policy_threshold::BatchedPolicyException],
) -> Vec<FallbackPolicyAssignment> {
    let mut out: Vec<FallbackPolicyAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_policy_assignment(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_fallback_policy_assignment(item: &FallbackPolicyAssignment) -> String {
    policy_threshold::strict_policy_threshold_bucket(item.weighted_value())
}
