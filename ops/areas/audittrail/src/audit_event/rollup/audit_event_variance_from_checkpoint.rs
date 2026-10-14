use super::*;

use std::collections::BTreeMap;

use crate::query::forecast as query_forecast;

pub fn summarize_query_checkpoints_for_variance(items: &[query_forecast::QueryCheckpoint]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_audit_event_variances_from_checkpoint(
    items: &[query_forecast::QueryCheckpoint],
) -> Vec<AuditEventVariance> {
    let mut out: Vec<AuditEventVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_audit_event_variance(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_audit_event_variance(item: &AuditEventVariance) -> String {
    query_forecast::classify_pending_query_quota(item.weighted_exposure())
}
