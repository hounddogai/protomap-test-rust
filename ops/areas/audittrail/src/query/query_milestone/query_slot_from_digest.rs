use super::*;

use std::collections::BTreeMap;

use crate::query::milestone as query_milestone;

pub fn summarize_query_digests_for_slot(items: &[query_milestone::QueryDigest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_query_slots_from_digest(items: &[query_milestone::QueryDigest]) -> Vec<QuerySlot> {
    let mut out: Vec<QuerySlot> = Vec::new();
    for item in items.iter() {
        let derived = new_query_slot(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_query_slot(item: &QuerySlot) -> String {
    query_milestone::query_variance_bucket(item.weighted_load())
}
