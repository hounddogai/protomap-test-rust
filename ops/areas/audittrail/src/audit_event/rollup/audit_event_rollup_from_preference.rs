use super::*;

use std::collections::BTreeMap;

use crate::retention::baseline as retention_baseline;

pub fn summarize_retention_preferences_for_rollup(
    items: &[retention_baseline::RetentionPreference],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_audit_event_rollups_from_preference(
    items: &[retention_baseline::RetentionPreference],
) -> Vec<AuditEventRollup> {
    let mut out: Vec<AuditEventRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_audit_event_rollup(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_audit_event_rollup(item: &AuditEventRollup) -> String {
    retention_baseline::normalize_retention_preference_code(&item.code)
}
