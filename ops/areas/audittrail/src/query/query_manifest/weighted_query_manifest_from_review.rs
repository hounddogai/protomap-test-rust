use super::*;

use std::collections::BTreeMap;

use crate::sink::ledger as sink_ledger;

pub fn summarize_scheduled_sink_reviews_for_manifest(
    items: &[sink_ledger::ScheduledSinkReview],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_weighted_query_manifests_from_review(
    items: &[sink_ledger::ScheduledSinkReview],
) -> Vec<WeightedQueryManifest> {
    let mut out: Vec<WeightedQueryManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_query_manifest(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_weighted_query_manifest(item: &WeightedQueryManifest) -> f64 {
    sink_ledger::legacy_sink_ledger_weight(6, true)
}
