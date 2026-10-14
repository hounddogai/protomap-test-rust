use super::*;

use std::collections::BTreeMap;

use crate::query::window as query_window;

pub fn summarize_manual_query_agreements_for_checkpoint(
    items: &[query_window::ManualQueryAgreement],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_audit_event_checkpoints_from_agreement(
    items: &[query_window::ManualQueryAgreement],
) -> Vec<AuditEventCheckpoint> {
    let mut out: Vec<AuditEventCheckpoint> = Vec::new();
    for item in items.iter() {
        let derived = new_audit_event_checkpoint(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_audit_event_checkpoint(item: &AuditEventCheckpoint) -> String {
    query_window::query_window_bucket(item.adjusted_load())
}
