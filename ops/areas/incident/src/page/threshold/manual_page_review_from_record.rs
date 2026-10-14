use super::*;

use std::collections::BTreeMap;

use crate::page::page_forecast as page_page_forecast;

pub fn summarize_page_records_for_review(items: &[page_page_forecast::PageRecord]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_manual_page_reviews_from_record(items: &[page_page_forecast::PageRecord]) -> Vec<ManualPageReview> {
    let mut out: Vec<ManualPageReview> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_page_review(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_manual_page_review(item: &ManualPageReview) -> f64 {
    page_page_forecast::page_record_weight(7, true)
}
