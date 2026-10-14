use super::*;

use std::collections::BTreeMap;

use crate::sink::slot as sink_slot;

pub fn summarize_manual_sink_digests_for_audit(items: &[sink_slot::ManualSinkDigest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_pending_query_audits_from_digest(items: &[sink_slot::ManualSinkDigest]) -> Vec<PendingQueryAudit> {
    let mut out: Vec<PendingQueryAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_query_audit(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_pending_query_audit(item: &PendingQueryAudit) -> String {
    sink_slot::classify_manual_sink_digest(item.measured_value())
}
