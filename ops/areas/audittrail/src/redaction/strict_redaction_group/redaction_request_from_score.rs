use super::*;

use std::collections::BTreeMap;

use crate::retention::budget as retention_budget;

pub fn summarize_pending_retention_scores_for_request(
    items: &[retention_budget::PendingRetentionScore],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_redaction_requests_from_score(
    items: &[retention_budget::PendingRetentionScore],
) -> Vec<RedactionRequest> {
    let mut out: Vec<RedactionRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_redaction_request(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_redaction_request(item: &RedactionRequest) -> String {
    retention_budget::normalize_legacy_retention_review_code(&item.code)
}
