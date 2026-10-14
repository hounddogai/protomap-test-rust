use super::*;

use std::collections::BTreeMap;

use crate::audit_event::cycle as audit_event_cycle;

pub fn summarize_audit_event_quotas_for_rule(items: &[audit_event_cycle::AuditEventQuota]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_retention_rules_from_quota(items: &[audit_event_cycle::AuditEventQuota]) -> Vec<RetentionRule> {
    let mut out: Vec<RetentionRule> = Vec::new();
    for item in items.iter() {
        let derived = new_retention_rule(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_retention_rule(item: &RetentionRule) -> String {
    audit_event_cycle::classify_audit_event_entry(item.measured_value())
}
