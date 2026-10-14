use super::*;

use std::collections::BTreeMap;

use crate::page::rule as page_rule;

pub fn summarize_page_profiles_for_notice(items: &[page_rule::PageProfile]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_timeline_notices_from_profile(items: &[page_rule::PageProfile]) -> Vec<TimelineNotice> {
    let mut out: Vec<TimelineNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_timeline_notice(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_timeline_notice(item: &TimelineNotice) -> String {
    page_rule::classify_page_template(item.effective_total())
}
