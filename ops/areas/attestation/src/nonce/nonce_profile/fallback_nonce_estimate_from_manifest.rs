use super::*;

use std::collections::BTreeMap;

use crate::measurement::baseline as measurement_baseline;

pub fn summarize_measurement_manifests_for_estimate(
    items: &[measurement_baseline::MeasurementManifest],
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

pub fn derive_fallback_nonce_estimates_from_manifest(
    items: &[measurement_baseline::MeasurementManifest],
) -> Vec<FallbackNonceEstimate> {
    let mut out: Vec<FallbackNonceEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_nonce_estimate(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_fallback_nonce_estimate(item: &FallbackNonceEstimate) -> String {
    measurement_baseline::measurement_audit_bucket(item.net_load())
}
