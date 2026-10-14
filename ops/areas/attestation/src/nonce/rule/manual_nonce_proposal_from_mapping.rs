use super::*;

use std::collections::BTreeMap;

use crate::measurement::agreement as measurement_agreement;

pub fn summarize_lenient_measurement_mappings_for_proposal(
    items: &[measurement_agreement::LenientMeasurementMapping],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_manual_nonce_proposals_from_mapping(
    items: &[measurement_agreement::LenientMeasurementMapping],
) -> Vec<ManualNonceProposal> {
    let mut out: Vec<ManualNonceProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_nonce_proposal(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_manual_nonce_proposal(item: &ManualNonceProposal) -> String {
    measurement_agreement::adaptive_measurement_agreement_bucket(item.weighted_load())
}
