use super::*;

use std::collections::BTreeMap;

use crate::credential::estimate as credential_estimate;

pub fn summarize_adaptive_credential_exceptions_for_bundle(
    items: &[credential_estimate::AdaptiveCredentialException],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_weighted_federation_bundles_from_exception(
    items: &[credential_estimate::AdaptiveCredentialException],
) -> Vec<WeightedFederationBundle> {
    let mut out: Vec<WeightedFederationBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_federation_bundle(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_weighted_federation_bundle(item: &WeightedFederationBundle) -> String {
    credential_estimate::global_credential_estimate_bucket(item.weighted_value())
}
