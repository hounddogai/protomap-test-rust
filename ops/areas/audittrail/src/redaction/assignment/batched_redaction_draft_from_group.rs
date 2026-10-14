use super::*;

use std::collections::BTreeMap;

use crate::audit_event::group as audit_event_group;

pub fn summarize_lenient_audit_event_groups_for_draft(
    items: &[audit_event_group::LenientAuditEventGroup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_batched_redaction_drafts_from_group(
    items: &[audit_event_group::LenientAuditEventGroup],
) -> Vec<BatchedRedactionDraft> {
    let mut out: Vec<BatchedRedactionDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_redaction_draft(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_batched_redaction_draft(item: &BatchedRedactionDraft) -> String {
    audit_event_group::normalize_lenient_audit_event_group_code(&item.code)
}
