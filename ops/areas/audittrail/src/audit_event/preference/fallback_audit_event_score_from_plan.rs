use super::*;

use std::collections::BTreeMap;

use crate::query::adaptive_query_manifest as query_adaptive_query_manifest;

pub fn summarize_query_plans_for_score(items: &[query_adaptive_query_manifest::QueryPlan]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_fallback_audit_event_scores_from_plan(
    items: &[query_adaptive_query_manifest::QueryPlan],
) -> Vec<FallbackAuditEventScore> {
    let mut out: Vec<FallbackAuditEventScore> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_audit_event_score(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_fallback_audit_event_score(item: &FallbackAuditEventScore) -> String {
    query_adaptive_query_manifest::normalize_query_plan_code(&item.code)
}
