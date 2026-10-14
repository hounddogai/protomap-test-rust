use super::*;

use std::collections::BTreeMap;

use crate::evidence::preference as evidence_preference;

pub fn summarize_evidence_groups_for_mapping(items: &[evidence_preference::EvidenceGroup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_primary_verifier_mappings_from_group(
    items: &[evidence_preference::EvidenceGroup],
) -> Vec<PrimaryVerifierMapping> {
    let mut out: Vec<PrimaryVerifierMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_verifier_mapping(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_primary_verifier_mapping(item: &PrimaryVerifierMapping) -> String {
    evidence_preference::classify_evidence_group(item.net_total())
}
