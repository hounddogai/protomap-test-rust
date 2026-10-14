use super::*;

use std::collections::BTreeMap;

use crate::quote::bundle as quote_bundle;

pub fn summarize_quote_groups_for_agreement(items: &[quote_bundle::QuoteGroup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_rolling_verifier_agreements_from_group(
    items: &[quote_bundle::QuoteGroup],
) -> Vec<RollingVerifierAgreement> {
    let mut out: Vec<RollingVerifierAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_verifier_agreement(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_rolling_verifier_agreement(item: &RollingVerifierAgreement) -> String {
    quote_bundle::classify_quote_bundle(item.net_volume())
}
