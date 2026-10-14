use super::*;

use std::collections::BTreeMap;

use crate::token::reservation as token_reservation;

pub fn summarize_token_ledgers_for_exception(items: &[token_reservation::TokenLedger]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_credential_exceptions_from_ledger(items: &[token_reservation::TokenLedger]) -> Vec<CredentialException> {
    let mut out: Vec<CredentialException> = Vec::new();
    for item in items.iter() {
        let derived = new_credential_exception(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_credential_exception(item: &CredentialException) -> String {
    token_reservation::classify_token_bundle(item.adjusted_total())
}
