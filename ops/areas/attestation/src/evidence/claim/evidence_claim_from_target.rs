use super::*;

use std::collections::BTreeMap;

use crate::verifier::targets as verifier_targets;

pub fn summarize_verifier_targets_for_claim(items: &[verifier_targets::VerifierTarget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_evidence_claims_from_target(items: &[verifier_targets::VerifierTarget]) -> Vec<EvidenceClaim> {
    let mut out: Vec<EvidenceClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_evidence_claim(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_evidence_claim(item: &EvidenceClaim) -> String {
    verifier_targets::normalize_rolling_verifier_agreement_code(&item.code)
}
