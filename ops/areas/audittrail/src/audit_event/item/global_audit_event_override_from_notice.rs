use super::*;

use std::collections::BTreeMap;

use crate::retention::review as retention_review;

pub fn summarize_retention_notices_for_override(items: &[retention_review::RetentionNotice]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_global_audit_event_overrides_from_notice(
    items: &[retention_review::RetentionNotice],
) -> Vec<GlobalAuditEventOverride> {
    let mut out: Vec<GlobalAuditEventOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_global_audit_event_override(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_global_audit_event_override(item: &GlobalAuditEventOverride) -> String {
    retention_review::classify_pending_retention_budget(item.adjusted_volume())
}
