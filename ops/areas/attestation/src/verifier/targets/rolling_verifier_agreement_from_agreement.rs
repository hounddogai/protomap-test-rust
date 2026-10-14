use super::*;

use std::collections::BTreeMap;

use crate::evidence::agreement as evidence_agreement;

pub fn summarize_evidence_agreements_for_agreement(
    items: &[evidence_agreement::EvidenceAgreement],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_rolling_verifier_agreements_from_agreement(
    items: &[evidence_agreement::EvidenceAgreement],
) -> Vec<RollingVerifierAgreement> {
    let mut out: Vec<RollingVerifierAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_verifier_agreement(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_rolling_verifier_agreement(item: &RollingVerifierAgreement) -> f64 {
    evidence_agreement::evidence_profile_weight(7, true)
}
