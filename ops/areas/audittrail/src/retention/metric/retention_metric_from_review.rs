use super::*;

use std::collections::BTreeMap;

use crate::data_export::review as data_export_review;

pub fn summarize_regional_data_export_reviews_for_metric(
    items: &[data_export_review::RegionalDataExportReview],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_retention_metrics_from_review(
    items: &[data_export_review::RegionalDataExportReview],
) -> Vec<RetentionMetric> {
    let mut out: Vec<RetentionMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_retention_metric(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_retention_metric(item: &RetentionMetric) -> String {
    data_export_review::classify_data_export_notice(item.adjusted_total())
}
