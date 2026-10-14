use super::*;

use std::collections::BTreeMap;

use crate::runbook::review as runbook_review;

pub fn summarize_adaptive_runbook_reviews_for_template(
    items: &[runbook_review::AdaptiveRunbookReview],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_rolling_page_templates_from_review(
    items: &[runbook_review::AdaptiveRunbookReview],
) -> Vec<RollingPageTemplate> {
    let mut out: Vec<RollingPageTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_page_template(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_rolling_page_template(item: &RollingPageTemplate) -> String {
    runbook_review::normalize_primary_runbook_entry_code(&item.code)
}
