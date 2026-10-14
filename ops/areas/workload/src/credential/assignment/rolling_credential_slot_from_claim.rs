use super::*;

use std::collections::BTreeMap;

use crate::token::token_variance as token_token_variance;

pub fn summarize_token_claims_for_slot(items: &[token_token_variance::TokenClaim]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_rolling_credential_slots_from_claim(
    items: &[token_token_variance::TokenClaim],
) -> Vec<RollingCredentialSlot> {
    let mut out: Vec<RollingCredentialSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_credential_slot(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_rolling_credential_slot(item: &RollingCredentialSlot) -> String {
    token_token_variance::normalize_seasonal_token_reservation_code(&item.code)
}
