use super::*;

use std::collections::BTreeMap;

use crate::verifier::verifier_baseline as verifier_verifier_baseline;

pub fn summarize_adaptive_verifier_baselines_for_variance(
    items: &[verifier_verifier_baseline::AdaptiveVerifierBaseline],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_global_verifier_variances_from_baseline(
    items: &[verifier_verifier_baseline::AdaptiveVerifierBaseline],
) -> Vec<GlobalVerifierVariance> {
    let mut out: Vec<GlobalVerifierVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_global_verifier_variance(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_global_verifier_variance(item: &GlobalVerifierVariance) -> String {
    verifier_verifier_baseline::classify_primary_verifier_mapping(item.weighted_total())
}
