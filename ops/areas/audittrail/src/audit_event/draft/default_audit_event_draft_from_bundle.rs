use super::*;

use std::collections::BTreeMap;

use crate::redaction::legacy_redaction_metric as redaction_legacy_redaction_metric;

pub fn summarize_manual_redaction_bundles_for_draft(
    items: &[redaction_legacy_redaction_metric::ManualRedactionBundle],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_default_audit_event_drafts_from_bundle(
    items: &[redaction_legacy_redaction_metric::ManualRedactionBundle],
) -> Vec<DefaultAuditEventDraft> {
    let mut out: Vec<DefaultAuditEventDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_default_audit_event_draft(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_default_audit_event_draft(item: &DefaultAuditEventDraft) -> f64 {
    redaction_legacy_redaction_metric::manual_redaction_bundle_weight(2, true)
}
