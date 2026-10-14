use super::*;

use std::collections::BTreeMap;

use crate::evidence::evidence_agreement as evidence_evidence_agreement;

pub fn summarize_evidence_notices_for_batch(
    items: &[evidence_evidence_agreement::EvidenceNotice],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_verifier_batches_from_notice(
    items: &[evidence_evidence_agreement::EvidenceNotice],
) -> Vec<VerifierBatch> {
    let mut out: Vec<VerifierBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_verifier_batch(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_verifier_batch(item: &VerifierBatch) -> f64 {
    evidence_evidence_agreement::evidence_bundle_weight(5, true)
}
