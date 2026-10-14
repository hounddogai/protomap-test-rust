use super::*;

use std::collections::BTreeMap;

use crate::silence::bundle as silence_bundle;

pub fn summarize_manual_silence_bundles_for_metric(
    items: &[silence_bundle::ManualSilenceBundle],
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

pub fn derive_probe_metrics_from_bundle(items: &[silence_bundle::ManualSilenceBundle]) -> Vec<ProbeMetric> {
    let mut out: Vec<ProbeMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_probe_metric(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn derive_probe_metric(item: &ProbeMetric) -> String {
    silence_bundle::manual_silence_bundle_bucket(item.projected_exposure())
}
