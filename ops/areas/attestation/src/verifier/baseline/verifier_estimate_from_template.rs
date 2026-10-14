use super::*;

use std::collections::BTreeMap;

use crate::evidence::template as evidence_template;

pub fn summarize_evidence_templates_for_estimate(
    items: &[evidence_template::EvidenceTemplate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_verifier_estimates_from_template(items: &[evidence_template::EvidenceTemplate]) -> Vec<VerifierEstimate> {
    let mut out: Vec<VerifierEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_verifier_estimate(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_verifier_estimate(item: &VerifierEstimate) -> String {
    evidence_template::classify_evidence_template(item.net_volume())
}
