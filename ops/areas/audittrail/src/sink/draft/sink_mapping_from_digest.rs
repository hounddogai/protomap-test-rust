use super::*;

use std::collections::BTreeMap;

use crate::query::milestone as query_milestone;

pub fn summarize_query_digests_for_mapping(items: &[query_milestone::QueryDigest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_sink_mappings_from_digest(items: &[query_milestone::QueryDigest]) -> Vec<SinkMapping> {
    let mut out: Vec<SinkMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_sink_mapping(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_sink_mapping(item: &SinkMapping) -> String {
    query_milestone::query_digest_bucket(item.weighted_value())
}
