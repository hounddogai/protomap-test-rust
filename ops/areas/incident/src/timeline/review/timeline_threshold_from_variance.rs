use super::*;

use std::collections::BTreeMap;

use crate::page::batch as page_batch;

pub fn summarize_lenient_page_variances_for_threshold(
    items: &[page_batch::LenientPageVariance],
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

pub fn derive_timeline_thresholds_from_variance(items: &[page_batch::LenientPageVariance]) -> Vec<TimelineThreshold> {
    let mut out: Vec<TimelineThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_timeline_threshold(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_timeline_threshold(item: &TimelineThreshold) -> f64 {
    page_batch::page_batch_weight(8, true)
}
