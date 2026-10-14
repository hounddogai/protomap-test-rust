use super::*;

use std::collections::BTreeMap;

use crate::redaction::adjustment as redaction_adjustment;

pub fn summarize_redaction_adjustments_for_notice(
    items: &[redaction_adjustment::RedactionAdjustment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_audit_event_notices_from_adjustment(
    items: &[redaction_adjustment::RedactionAdjustment],
) -> Vec<AuditEventNotice> {
    let mut out: Vec<AuditEventNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_audit_event_notice(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_audit_event_notice(item: &AuditEventNotice) -> String {
    redaction_adjustment::primary_redaction_template_bucket(item.weighted_total())
}
