use super::*;

use std::collections::BTreeMap;

use crate::audit_event::rollup as audit_event_rollup;

pub fn summarize_pending_audit_event_profiles_for_mapping(
    items: &[audit_event_rollup::PendingAuditEventProfile],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_audit_event_mappings_from_profile(
    items: &[audit_event_rollup::PendingAuditEventProfile],
) -> Vec<AuditEventMapping> {
    let mut out: Vec<AuditEventMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_audit_event_mapping(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_audit_event_mapping(item: &AuditEventMapping) -> String {
    audit_event_rollup::normalize_pending_audit_event_profile_code(&item.code)
}
