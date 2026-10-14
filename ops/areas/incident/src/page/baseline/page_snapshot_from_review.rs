use super::*;

use std::collections::BTreeMap;

use crate::postmortem::adjustment as postmortem_adjustment;

pub fn summarize_postmortem_reviews_for_snapshot(
    items: &[postmortem_adjustment::PostmortemReview],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_page_snapshots_from_review(items: &[postmortem_adjustment::PostmortemReview]) -> Vec<PageSnapshot> {
    let mut out: Vec<PageSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_page_snapshot(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_page_snapshot(item: &PageSnapshot) -> String {
    postmortem_adjustment::normalize_manual_postmortem_profile_code(&item.code)
}
