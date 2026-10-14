use super::*;

use std::collections::BTreeMap;

use crate::query::manifest as query_manifest;

pub fn summarize_query_manifests_for_ledger(items: &[query_manifest::QueryManifest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_sink_ledgers_from_manifest(items: &[query_manifest::QueryManifest]) -> Vec<SinkLedger> {
    let mut out: Vec<SinkLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_sink_ledger(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_sink_ledger(item: &SinkLedger) -> String {
    query_manifest::normalize_query_record_code(&item.code)
}
