use super::*;

use std::collections::BTreeMap;

use crate::sink::sink_bundle as sink_sink_bundle;

pub fn summarize_global_sink_bundles_for_score(items: &[sink_sink_bundle::GlobalSinkBundle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_adaptive_audit_event_scores_from_bundle(
    items: &[sink_sink_bundle::GlobalSinkBundle],
) -> Vec<AdaptiveAuditEventScore> {
    let mut out: Vec<AdaptiveAuditEventScore> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_audit_event_score(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_adaptive_audit_event_score(item: &AdaptiveAuditEventScore) -> String {
    sink_sink_bundle::normalize_global_sink_bundle_code(&item.code)
}
