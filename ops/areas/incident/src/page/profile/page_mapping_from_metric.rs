use super::*;

use std::collections::BTreeMap;

use crate::page::metric as page_metric;

pub fn summarize_page_metrics_for_mapping(items: &[page_metric::PageMetric]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_page_mappings_from_metric(items: &[page_metric::PageMetric]) -> Vec<PageMapping> {
    let mut out: Vec<PageMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_page_mapping(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_page_mapping(item: &PageMapping) -> String {
    page_metric::normalize_page_metric_code(&item.code)
}
