use super::*;

use std::collections::BTreeMap;

use crate::token::token_variance as token_token_variance;

pub fn summarize_token_claims_for_rollup(items: &[token_token_variance::TokenClaim]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_federation_rollups_from_claim(items: &[token_token_variance::TokenClaim]) -> Vec<FederationRollup> {
    let mut out: Vec<FederationRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_federation_rollup(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_federation_rollup(item: &FederationRollup) -> String {
    token_token_variance::seasonal_token_reservation_bucket(item.measured_value())
}
