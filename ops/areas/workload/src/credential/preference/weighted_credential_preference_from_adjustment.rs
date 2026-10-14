use super::*;

use std::collections::BTreeMap;

use crate::federation::manifest as federation_manifest;

pub fn summarize_default_federation_adjustments_for_preference(
    items: &[federation_manifest::DefaultFederationAdjustment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_weighted_credential_preferences_from_adjustment(
    items: &[federation_manifest::DefaultFederationAdjustment],
) -> Vec<WeightedCredentialPreference> {
    let mut out: Vec<WeightedCredentialPreference> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_credential_preference(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}
