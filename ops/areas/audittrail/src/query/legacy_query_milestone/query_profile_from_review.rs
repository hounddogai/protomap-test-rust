use super::*;

use std::collections::BTreeMap;

use crate::query::query_milestone as query_query_milestone;

pub fn summarize_query_reviews_for_profile(items: &[query_query_milestone::QueryReview]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_query_profiles_from_review(items: &[query_query_milestone::QueryReview]) -> Vec<QueryProfile> {
    let mut out: Vec<QueryProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_query_profile(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_query_profile(item: &QueryProfile) -> String {
    query_query_milestone::query_rollup_bucket(item.effective_total())
}
