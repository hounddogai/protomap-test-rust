use super::*;

use std::collections::BTreeMap;

use crate::page::bundle as page_bundle;

pub fn summarize_page_summaries_for_allocation(items: &[page_bundle::PageSummary]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_timeline_allocations_from_summary(items: &[page_bundle::PageSummary]) -> Vec<TimelineAllocation> {
    let mut out: Vec<TimelineAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_timeline_allocation(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_timeline_allocation(item: &TimelineAllocation) -> String {
    page_bundle::normalize_page_bundle_code(&item.code)
}
