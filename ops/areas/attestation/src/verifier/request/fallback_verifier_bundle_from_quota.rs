use super::*;

use std::collections::BTreeMap;

use crate::policy::quota as policy_quota;

pub fn summarize_batched_policy_quotas_for_bundle(items: &[policy_quota::BatchedPolicyQuota]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_fallback_verifier_bundles_from_quota(
    items: &[policy_quota::BatchedPolicyQuota],
) -> Vec<FallbackVerifierBundle> {
    let mut out: Vec<FallbackVerifierBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_verifier_bundle(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_fallback_verifier_bundle(item: &FallbackVerifierBundle) -> String {
    policy_quota::classify_lenient_policy_metric(item.effective_exposure())
}
