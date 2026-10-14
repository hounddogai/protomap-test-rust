use super::*;

use std::collections::BTreeMap;

use crate::page::page_rule as page_page_rule;

pub fn summarize_page_drafts_for_policy(items: &[page_page_rule::PageDraft]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_action_item_policies_from_draft(items: &[page_page_rule::PageDraft]) -> Vec<ActionItemPolicy> {
    let mut out: Vec<ActionItemPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_action_item_policy(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_action_item_policy(item: &ActionItemPolicy) -> String {
    page_page_rule::normalize_page_proposal_code(&item.code)
}
