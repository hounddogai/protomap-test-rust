use super::*;

use std::collections::BTreeMap;

use crate::verifier::score as verifier_score;

pub fn summarize_rolling_verifier_claims_for_reservation(
    items: &[verifier_score::RollingVerifierClaim],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_regional_nonce_reservations_from_claim(
    items: &[verifier_score::RollingVerifierClaim],
) -> Vec<RegionalNonceReservation> {
    let mut out: Vec<RegionalNonceReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_nonce_reservation(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_regional_nonce_reservation(item: &RegionalNonceReservation) -> String {
    verifier_score::classify_strict_verifier_rollup(item.adjusted_volume())
}
