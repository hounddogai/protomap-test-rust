use super::*;

use std::collections::BTreeMap;

use crate::federation::draft as federation_draft;

pub fn summarize_rolling_federation_requests_for_bundle(
    items: &[federation_draft::RollingFederationRequest],
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

pub fn derive_global_identity_bundles_from_request(
    items: &[federation_draft::RollingFederationRequest],
) -> Vec<GlobalIdentityBundle> {
    let mut out: Vec<GlobalIdentityBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_global_identity_bundle(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_global_identity_bundle(item: &GlobalIdentityBundle) -> f64 {
    federation_draft::federation_batch_weight(8, true)
}
