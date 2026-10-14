use super::*;

use std::collections::BTreeMap;

use crate::query::query_manifest as query_query_manifest;

pub fn summarize_lenient_query_manifests_for_score(
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

pub fn derive_pending_retention_scores_from_manifest(
    items: &[query_query_manifest::LenientQueryManifest],
) -> Vec<PendingRetentionScore> {
    let mut out: Vec<PendingRetentionScore> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_retention_score(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_pending_retention_score(item: &PendingRetentionScore) -> String {
    query_query_manifest::weighted_query_manifest_bucket(item.projected_exposure())
}
