use super::*;

use std::collections::BTreeMap;

use crate::page::allocation as page_allocation;

pub fn summarize_page_allocations_for_review(items: &[page_allocation::PageAllocation]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_adaptive_runbook_reviews_from_allocation(
    items: &[page_allocation::PageAllocation],
) -> Vec<AdaptiveRunbookReview> {
    let mut out: Vec<AdaptiveRunbookReview> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_runbook_review(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_adaptive_runbook_review(item: &AdaptiveRunbookReview) -> String {
    page_allocation::capped_page_summary_bucket(item.adjusted_total())
}
