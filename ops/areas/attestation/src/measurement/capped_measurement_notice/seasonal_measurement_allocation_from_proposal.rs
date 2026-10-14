use super::*;

use std::collections::BTreeMap;

use crate::nonce::nonce_proposal as nonce_nonce_proposal;

pub fn summarize_legacy_nonce_proposals_for_allocation(
    items: &[nonce_nonce_proposal::LegacyNonceProposal],
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

pub fn derive_seasonal_measurement_allocations_from_proposal(
    items: &[nonce_nonce_proposal::LegacyNonceProposal],
) -> Vec<SeasonalMeasurementAllocation> {
    let mut out: Vec<SeasonalMeasurementAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_measurement_allocation(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_seasonal_measurement_allocation(item: &SeasonalMeasurementAllocation) -> String {
    nonce_nonce_proposal::classify_legacy_nonce_proposal(item.measured_value())
}
