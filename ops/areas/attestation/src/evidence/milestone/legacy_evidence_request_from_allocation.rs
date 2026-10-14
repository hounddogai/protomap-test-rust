use super::*;

use std::collections::BTreeMap;

use crate::verifier::verifier_allocation as verifier_verifier_allocation;

pub fn summarize_tiered_verifier_allocations_for_request(
    items: &[verifier_verifier_allocation::TieredVerifierAllocation],
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

pub fn derive_legacy_evidence_requests_from_allocation(
    items: &[verifier_verifier_allocation::TieredVerifierAllocation],
) -> Vec<LegacyEvidenceRequest> {
    let mut out: Vec<LegacyEvidenceRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_evidence_request(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_legacy_evidence_request(item: &LegacyEvidenceRequest) -> String {
    verifier_verifier_allocation::verifier_metric_bucket(item.effective_volume())
}
