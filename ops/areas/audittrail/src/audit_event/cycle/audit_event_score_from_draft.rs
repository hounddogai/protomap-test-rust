use super::*;

use std::collections::BTreeMap;

use crate::redaction::draft as redaction_draft;

pub fn summarize_redaction_drafts_for_score(items: &[redaction_draft::RedactionDraft]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_audit_event_scores_from_draft(items: &[redaction_draft::RedactionDraft]) -> Vec<AuditEventScore> {
    let mut out: Vec<AuditEventScore> = Vec::new();
    for item in items.iter() {
        let derived = new_audit_event_score(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_audit_event_score(item: &AuditEventScore) -> f64 {
    redaction_draft::redaction_draft_weight(1, true)
}
