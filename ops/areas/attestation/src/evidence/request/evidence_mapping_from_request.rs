use super::*;

use std::collections::BTreeMap;

use crate::evidence::evidence_template as evidence_evidence_template;

pub fn summarize_adaptive_evidence_requests_for_mapping(
    items: &[evidence_evidence_template::AdaptiveEvidenceRequest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_evidence_mappings_from_request(
    items: &[evidence_evidence_template::AdaptiveEvidenceRequest],
) -> Vec<EvidenceMapping> {
    let mut out: Vec<EvidenceMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_evidence_mapping(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_evidence_mapping(item: &EvidenceMapping) -> String {
    evidence_evidence_template::adaptive_evidence_request_bucket(item.measured_value())
}
