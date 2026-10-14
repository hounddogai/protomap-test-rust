use super::*;

use std::collections::BTreeMap;

use crate::audit_event::adjustment as audit_event_adjustment;

pub fn summarize_audit_event_adjustments_for_window(
    items: &[audit_event_adjustment::AuditEventAdjustment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_retention_windows_from_adjustment(
    items: &[audit_event_adjustment::AuditEventAdjustment],
) -> Vec<RetentionWindow> {
    let mut out: Vec<RetentionWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_retention_window(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_retention_window(item: &RetentionWindow) -> String {
    audit_event_adjustment::audit_event_adjustment_bucket(item.effective_volume())
}
