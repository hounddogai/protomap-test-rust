use super::*;

use std::collections::BTreeMap;

use crate::page::page_rule as page_page_rule;

pub fn summarize_strict_page_rules_for_override(items: &[page_page_rule::StrictPageRule]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_postmortem_overrides_from_rule(items: &[page_page_rule::StrictPageRule]) -> Vec<PostmortemOverride> {
    let mut out: Vec<PostmortemOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_postmortem_override(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_postmortem_override(item: &PostmortemOverride) -> String {
    page_page_rule::normalize_page_group_code(&item.code)
}
