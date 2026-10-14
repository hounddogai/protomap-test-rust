use super::*;

use std::collections::BTreeMap;

use crate::sink::quota as sink_quota;

pub fn summarize_sink_exceptions_for_exception(items: &[sink_quota::SinkException]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_audit_event_exceptions_from_exception(items: &[sink_quota::SinkException]) -> Vec<AuditEventException> {
    let mut out: Vec<AuditEventException> = Vec::new();
    for item in items.iter() {
        let derived = new_audit_event_exception(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_audit_event_exception(item: &AuditEventException) -> String {
    sink_quota::normalize_sink_quota_code(&item.code)
}
