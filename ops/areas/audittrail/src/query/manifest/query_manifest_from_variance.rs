use super::*;

use std::collections::BTreeMap;

use crate::sink::quota as sink_quota;

pub fn summarize_sink_variances_for_manifest(items: &[sink_quota::SinkVariance]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_query_manifests_from_variance(items: &[sink_quota::SinkVariance]) -> Vec<QueryManifest> {
    let mut out: Vec<QueryManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_query_manifest(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_query_manifest(item: &QueryManifest) -> String {
    sink_quota::classify_sink_score(item.adjusted_load())
}
