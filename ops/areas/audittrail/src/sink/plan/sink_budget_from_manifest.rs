use super::*;

use std::collections::BTreeMap;

use crate::query::query_manifest as query_query_manifest;

pub fn summarize_lenient_query_manifests_for_budget(
    items: &[query_query_manifest::LenientQueryManifest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_sink_budgets_from_manifest(items: &[query_query_manifest::LenientQueryManifest]) -> Vec<SinkBudget> {
    let mut out: Vec<SinkBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_sink_budget(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_sink_budget(item: &SinkBudget) -> String {
    query_query_manifest::normalize_lenient_query_manifest_code(&item.code)
}
