use super::*;

use std::collections::BTreeMap;

use crate::audit_event::slot as audit_event_slot;

pub fn summarize_adaptive_audit_event_entries_for_item(
    items: &[audit_event_slot::AdaptiveAuditEventEntry],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_audit_event_items_from_entry(items: &[audit_event_slot::AdaptiveAuditEventEntry]) -> Vec<AuditEventItem> {
    let mut out: Vec<AuditEventItem> = Vec::new();
    for item in items.iter() {
        let derived = new_audit_event_item(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_audit_event_item(item: &AuditEventItem) -> f64 {
    audit_event_slot::rolling_audit_event_segment_weight(3, true)
}
