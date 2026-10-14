use super::*;

use std::collections::BTreeMap;

use crate::page::bundle as page_bundle;

pub fn summarize_page_reviews_for_rule(items: &[page_bundle::PageReview]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_action_item_rules_from_review(items: &[page_bundle::PageReview]) -> Vec<ActionItemRule> {
    let mut out: Vec<ActionItemRule> = Vec::new();
    for item in items.iter() {
        let derived = new_action_item_rule(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_action_item_rule(item: &ActionItemRule) -> String {
    page_bundle::normalize_page_bundle_code(&item.code)
}
