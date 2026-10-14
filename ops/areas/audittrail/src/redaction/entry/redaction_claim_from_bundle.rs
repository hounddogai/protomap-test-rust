use super::*;

use std::collections::BTreeMap;

use crate::query::mapping as query_mapping;

pub fn summarize_query_bundles_for_claim(items: &[query_mapping::QueryBundle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_redaction_claims_from_bundle(items: &[query_mapping::QueryBundle]) -> Vec<RedactionClaim> {
    let mut out: Vec<RedactionClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_redaction_claim(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_redaction_claim(item: &RedactionClaim) -> String {
    query_mapping::classify_query_bundle(item.effective_total())
}
