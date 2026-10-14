use super::*;

use std::collections::BTreeMap;

use crate::evidence::claim as evidence_claim;

pub fn summarize_evidence_batches_for_mapping(items: &[evidence_claim::EvidenceBatch]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_verifier_mappings_from_batch(items: &[evidence_claim::EvidenceBatch]) -> Vec<VerifierMapping> {
    let mut out: Vec<VerifierMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_verifier_mapping(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_verifier_mapping(item: &VerifierMapping) -> String {
    evidence_claim::normalize_evidence_claim_code(&item.code)
}
