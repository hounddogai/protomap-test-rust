use super::*;

use std::collections::BTreeMap;

use crate::evidence::evidence_template as evidence_evidence_template;

pub fn summarize_scheduled_evidence_templates_for_draft(
    items: &[evidence_evidence_template::ScheduledEvidenceTemplate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_nonce_drafts_from_template(
    items: &[evidence_evidence_template::ScheduledEvidenceTemplate],
) -> Vec<NonceDraft> {
    let mut out: Vec<NonceDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_nonce_draft(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_nonce_draft(item: &NonceDraft) -> String {
    evidence_evidence_template::classify_scheduled_evidence_template(item.measured_value())
}
