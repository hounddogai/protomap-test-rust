use super::*;

use std::collections::BTreeMap;

use crate::query::legacy_query_milestone as query_legacy_query_milestone;

pub fn summarize_scheduled_query_reviews_for_audit(
    items: &[query_legacy_query_milestone::ScheduledQueryReview],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_query_audits_from_review(
    items: &[query_legacy_query_milestone::ScheduledQueryReview],
) -> Vec<QueryAudit> {
    let mut out: Vec<QueryAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_query_audit(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_query_audit(item: &QueryAudit) -> String {
    query_legacy_query_milestone::scheduled_query_review_bucket(item.net_value())
}
