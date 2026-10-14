use super::*;

use std::collections::BTreeMap;

use crate::audit_event::preference as audit_event_preference;

pub fn summarize_fallback_audit_event_items_for_group(
    items: &[audit_event_preference::FallbackAuditEventItem],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_retention_groups_from_item(
    items: &[audit_event_preference::FallbackAuditEventItem],
) -> Vec<RetentionGroup> {
    let mut out: Vec<RetentionGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_retention_group(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn derive_retention_group(item: &RetentionGroup) -> String {
    audit_event_preference::normalize_seasonal_audit_event_preference_code(&item.code)
}
