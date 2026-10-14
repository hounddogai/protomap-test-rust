use super::*;

use std::collections::BTreeMap;

use crate::audit_event::adjustment as audit_event_adjustment;

pub fn summarize_audit_event_digests_for_variance(
    items: &[audit_event_adjustment::AuditEventDigest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_sink_variances_from_digest(items: &[audit_event_adjustment::AuditEventDigest]) -> Vec<SinkVariance> {
    let mut out: Vec<SinkVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_sink_variance(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_sink_variance(item: &SinkVariance) -> String {
    audit_event_adjustment::audit_event_adjustment_bucket(item.measured_value())
}
