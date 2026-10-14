use super::*;

use std::collections::BTreeMap;

use crate::verifier::verifier_allocation as verifier_verifier_allocation;

pub fn summarize_legacy_verifier_manifests_for_forecast(
    items: &[verifier_verifier_allocation::LegacyVerifierManifest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_legacy_nonce_forecasts_from_manifest(
    items: &[verifier_verifier_allocation::LegacyVerifierManifest],
) -> Vec<LegacyNonceForecast> {
    let mut out: Vec<LegacyNonceForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_nonce_forecast(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_legacy_nonce_forecast(item: &LegacyNonceForecast) -> String {
    verifier_verifier_allocation::verifier_metric_bucket(item.projected_value())
}
