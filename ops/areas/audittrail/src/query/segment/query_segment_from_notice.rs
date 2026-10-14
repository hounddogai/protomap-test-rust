use super::*;

use std::collections::BTreeMap;

use crate::data_export::review as data_export_review;

pub fn summarize_data_export_notices_for_segment(
    items: &[data_export_review::DataExportNotice],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_query_segments_from_notice(items: &[data_export_review::DataExportNotice]) -> Vec<QuerySegment> {
    let mut out: Vec<QuerySegment> = Vec::new();
    for item in items.iter() {
        let derived = new_query_segment(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_query_segment(item: &QuerySegment) -> String {
    data_export_review::regional_data_export_review_bucket(item.projected_total())
}
