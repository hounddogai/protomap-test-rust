use super::*;

use std::collections::BTreeMap;

use crate::sink::sink_bundle as sink_sink_bundle;

pub fn summarize_global_sink_bundles_for_ledger(items: &[sink_sink_bundle::GlobalSinkBundle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_query_ledgers_from_bundle(items: &[sink_sink_bundle::GlobalSinkBundle]) -> Vec<QueryLedger> {
    let mut out: Vec<QueryLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_query_ledger(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_query_ledger(item: &QueryLedger) -> String {
    sink_sink_bundle::normalize_global_sink_bundle_code(&item.code)
}
