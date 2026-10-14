use super::*;

use std::collections::BTreeMap;

use crate::verifier::summary as verifier_summary;

pub fn summarize_regional_verifier_summaries_for_estimate(
    items: &[verifier_summary::RegionalVerifierSummary],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_fallback_measurement_estimates_from_summary(
    items: &[verifier_summary::RegionalVerifierSummary],
) -> Vec<FallbackMeasurementEstimate> {
    let mut out: Vec<FallbackMeasurementEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_measurement_estimate(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_fallback_measurement_estimate(item: &FallbackMeasurementEstimate) -> String {
    verifier_summary::classify_manual_verifier_reservation(item.net_volume())
}
