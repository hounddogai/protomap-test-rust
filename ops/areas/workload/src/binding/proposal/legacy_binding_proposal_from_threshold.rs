use super::*;

use std::collections::BTreeMap;

use crate::credential::default_credential_score as credential_default_credential_score;

pub fn summarize_weighted_credential_thresholds_for_proposal(
    items: &[credential_default_credential_score::WeightedCredentialThreshold],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_legacy_binding_proposals_from_threshold(
    items: &[credential_default_credential_score::WeightedCredentialThreshold],
) -> Vec<LegacyBindingProposal> {
    let mut out: Vec<LegacyBindingProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_binding_proposal(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_legacy_binding_proposal(item: &LegacyBindingProposal) -> f64 {
    credential_default_credential_score::weighted_credential_threshold_weight(7, true)
}
