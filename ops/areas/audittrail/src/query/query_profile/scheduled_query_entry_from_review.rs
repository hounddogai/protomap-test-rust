use super::*;

use std::collections::BTreeMap;

use crate::data_export::export_review as data_export_export_review;

pub fn summarize_scheduled_data_export_reviews_for_entry(
    items: &[data_export_export_review::ScheduledDataExportReview],
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

pub fn derive_scheduled_query_entries_from_review(
    items: &[data_export_export_review::ScheduledDataExportReview],
) -> Vec<ScheduledQueryEntry> {
    let mut out: Vec<ScheduledQueryEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_query_entry(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_scheduled_query_entry(item: &ScheduledQueryEntry) -> f64 {
    data_export_export_review::adaptive_data_export_metric_weight(6, true)
}
