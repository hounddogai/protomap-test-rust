use super::*;

use std::collections::BTreeMap;

use crate::verifier::budget as verifier_budget;

pub fn summarize_verifier_agreements_for_agreement(
    items: &[verifier_budget::VerifierAgreement],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_quote_agreements_from_agreement(items: &[verifier_budget::VerifierAgreement]) -> Vec<QuoteAgreement> {
    let mut out: Vec<QuoteAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_quote_agreement(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_quote_agreement(item: &QuoteAgreement) -> String {
    verifier_budget::normalize_verifier_ledger_code(&item.code)
}
