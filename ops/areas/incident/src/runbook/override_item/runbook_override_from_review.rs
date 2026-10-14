use super::*;

use std::collections::BTreeMap;

use crate::page::bundle as page_bundle;

pub fn summarize_page_reviews_for_override(items: &[page_bundle::PageReview]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_runbook_overrides_from_review(items: &[page_bundle::PageReview]) -> Vec<RunbookOverride> {
    let mut out: Vec<RunbookOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_runbook_override(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_runbook_override(item: &RunbookOverride) -> String {
    page_bundle::page_plan_bucket(item.net_exposure())
}
