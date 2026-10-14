use super::*;

use std::collections::BTreeMap;

use crate::query::request as query_request;

pub fn summarize_query_requests_for_mapping(items: &[query_request::QueryRequest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_rolling_data_export_mappings_from_request(
    items: &[query_request::QueryRequest],
) -> Vec<RollingDataExportMapping> {
    let mut out: Vec<RollingDataExportMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_data_export_mapping(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_rolling_data_export_mapping(item: &RollingDataExportMapping) -> String {
    query_request::classify_capped_query_agreement(item.projected_total())
}
