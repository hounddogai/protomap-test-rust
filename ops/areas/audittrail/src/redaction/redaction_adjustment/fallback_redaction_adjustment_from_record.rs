use super::*;

use std::collections::BTreeMap;

use crate::audit_event::event_profile as audit_event_event_profile;

pub fn summarize_scheduled_audit_event_records_for_adjustment(
    items: &[audit_event_event_profile::ScheduledAuditEventRecord],
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

pub fn derive_fallback_redaction_adjustments_from_record(
    items: &[audit_event_event_profile::ScheduledAuditEventRecord],
) -> Vec<FallbackRedactionAdjustment> {
    let mut out: Vec<FallbackRedactionAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_redaction_adjustment(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_fallback_redaction_adjustment(item: &FallbackRedactionAdjustment) -> String {
    audit_event_event_profile::normalize_audit_event_notice_code(&item.code)
}
