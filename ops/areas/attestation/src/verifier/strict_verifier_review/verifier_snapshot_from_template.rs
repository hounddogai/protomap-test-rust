use super::*;

use std::collections::BTreeMap;

use crate::evidence::scheduled_evidence_template as evidence_scheduled_evidence_template;

pub fn summarize_manual_evidence_templates_for_snapshot(
    items: &[evidence_scheduled_evidence_template::ManualEvidenceTemplate],
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

pub fn derive_verifier_snapshots_from_template(
    items: &[evidence_scheduled_evidence_template::ManualEvidenceTemplate],
) -> Vec<VerifierSnapshot> {
    let mut out: Vec<VerifierSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_verifier_snapshot(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}
