use super::*;

use std::collections::BTreeMap;

use crate::verifier::request as verifier_request;

pub fn summarize_fallback_verifier_bundles_for_target(
    items: &[verifier_request::FallbackVerifierBundle],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_quote_targets_from_bundle(items: &[verifier_request::FallbackVerifierBundle]) -> Vec<QuoteTarget> {
    let mut out: Vec<QuoteTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_quote_target(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_quote_target(item: &QuoteTarget) -> String {
    verifier_request::classify_fallback_verifier_bundle(item.projected_exposure())
}
