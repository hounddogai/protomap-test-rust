use super::*;

use std::collections::BTreeMap;

use crate::page::assignment as page_assignment;

pub fn summarize_page_assignments_for_review(items: &[page_assignment::PageAssignment]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_seasonal_action_item_reviews_from_assignment(
    items: &[page_assignment::PageAssignment],
) -> Vec<SeasonalActionItemReview> {
    let mut out: Vec<SeasonalActionItemReview> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_action_item_review(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_seasonal_action_item_review(item: &SeasonalActionItemReview) -> String {
    page_assignment::normalize_page_agreement_code(&item.code)
}
