use super::*;

use std::collections::BTreeMap;

use crate::retention::draft as retention_draft;

pub fn summarize_capped_retention_windows_for_request(
    items: &[retention_draft::CappedRetentionWindow],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_query_requests_from_window(items: &[retention_draft::CappedRetentionWindow]) -> Vec<QueryRequest> {
    let mut out: Vec<QueryRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_query_request(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_query_request(item: &QueryRequest) -> f64 {
    retention_draft::capped_retention_window_weight(4, true)
}
