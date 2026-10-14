use super::*;

use std::collections::BTreeMap;

use crate::data_export::metric as data_export_metric;

pub fn summarize_fallback_data_export_rules_for_request(
    items: &[data_export_metric::FallbackDataExportRule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_query_requests_from_rule(items: &[data_export_metric::FallbackDataExportRule]) -> Vec<QueryRequest> {
    let mut out: Vec<QueryRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_query_request(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_query_request(item: &QueryRequest) -> f64 {
    data_export_metric::data_export_override_weight(3, true)
}
