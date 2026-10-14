use super::*;

use std::collections::BTreeMap;

use crate::retention::retention_metric as retention_retention_metric;

pub fn summarize_primary_retention_metrics_for_manifest(
    items: &[retention_retention_metric::PrimaryRetentionMetric],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_tiered_query_manifests_from_metric(
    items: &[retention_retention_metric::PrimaryRetentionMetric],
) -> Vec<TieredQueryManifest> {
    let mut out: Vec<TieredQueryManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_query_manifest(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_tiered_query_manifest(item: &TieredQueryManifest) -> String {
    retention_retention_metric::primary_retention_metric_bucket(item.weighted_exposure())
}
