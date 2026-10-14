use super::*;

use std::collections::BTreeMap;

use crate::verifier::request as verifier_request;

pub fn summarize_manual_verifier_assignments_for_draft(
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

pub fn derive_evidence_drafts_from_assignment(
    items: &[verifier_request::ManualVerifierAssignment],
) -> Vec<EvidenceDraft> {
    let mut out: Vec<EvidenceDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_evidence_draft(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_evidence_draft(item: &EvidenceDraft) -> String {
    verifier_request::fallback_verifier_bundle_bucket(item.net_exposure())
}
