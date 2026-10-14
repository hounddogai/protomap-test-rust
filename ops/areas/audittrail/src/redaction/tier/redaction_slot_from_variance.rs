use super::*;

use std::collections::BTreeMap;

use crate::query::milestone as query_milestone;

pub fn summarize_query_variances_for_slot(items: &[query_milestone::QueryVariance]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_redaction_slots_from_variance(items: &[query_milestone::QueryVariance]) -> Vec<RedactionSlot> {
    let mut out: Vec<RedactionSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_redaction_slot(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_redaction_slot(item: &RedactionSlot) -> String {
    query_milestone::query_variance_bucket(item.projected_volume())
}
