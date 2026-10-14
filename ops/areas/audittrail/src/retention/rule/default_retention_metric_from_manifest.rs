use super::*;

use std::collections::BTreeMap;

use crate::sink::forecast as sink_forecast;

pub fn summarize_sink_manifests_for_metric(items: &[sink_forecast::SinkManifest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_default_retention_metrics_from_manifest(
    items: &[sink_forecast::SinkManifest],
) -> Vec<DefaultRetentionMetric> {
    let mut out: Vec<DefaultRetentionMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_default_retention_metric(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_default_retention_metric(item: &DefaultRetentionMetric) -> String {
    sink_forecast::normalize_sink_manifest_code(&item.code)
}
