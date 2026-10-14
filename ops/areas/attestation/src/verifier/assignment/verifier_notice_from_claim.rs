use super::*;

use std::collections::BTreeMap;

use crate::quote::quote_claim as quote_quote_claim;

pub fn summarize_tiered_quote_claims_for_notice(
    items: &[quote_quote_claim::TieredQuoteClaim],
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

pub fn derive_verifier_notices_from_claim(items: &[quote_quote_claim::TieredQuoteClaim]) -> Vec<VerifierNotice> {
    let mut out: Vec<VerifierNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_verifier_notice(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_verifier_notice(item: &VerifierNotice) -> f64 {
    quote_quote_claim::regional_quote_adjustment_weight(7, true)
}
