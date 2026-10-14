use super::*;

use std::collections::BTreeMap;

use crate::evidence::seasonal_evidence_agreement as evidence_seasonal_evidence_agreement;

pub fn summarize_fallback_evidence_agreements_for_claim(
    items: &[evidence_seasonal_evidence_agreement::FallbackEvidenceAgreement],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_verifier_claims_from_agreement(
    items: &[evidence_seasonal_evidence_agreement::FallbackEvidenceAgreement],
) -> Vec<VerifierClaim> {
    let mut out: Vec<VerifierClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_verifier_claim(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_verifier_claim(item: &VerifierClaim) -> String {
    evidence_seasonal_evidence_agreement::normalize_regional_evidence_manifest_code(&item.code)
}
