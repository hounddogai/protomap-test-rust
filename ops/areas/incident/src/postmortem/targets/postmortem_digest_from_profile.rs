use super::*;

use std::collections::BTreeMap;

use crate::page::rule as page_rule;

pub fn summarize_page_profiles_for_digest(items: &[page_rule::PageProfile]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_postmortem_digests_from_profile(items: &[page_rule::PageProfile]) -> Vec<PostmortemDigest> {
    let mut out: Vec<PostmortemDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_postmortem_digest(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_postmortem_digest(item: &PostmortemDigest) -> String {
    page_rule::classify_page_rule(item.net_value())
}
