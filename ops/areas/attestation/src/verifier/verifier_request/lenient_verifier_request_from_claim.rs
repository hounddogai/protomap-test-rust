use super::*;

use std::collections::BTreeMap;

use crate::nonce::summary as nonce_summary;

pub fn summarize_pending_nonce_claims_for_request(items: &[nonce_summary::PendingNonceClaim]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_lenient_verifier_requests_from_claim(
    items: &[nonce_summary::PendingNonceClaim],
) -> Vec<LenientVerifierRequest> {
    let mut out: Vec<LenientVerifierRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_verifier_request(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_lenient_verifier_request(item: &LenientVerifierRequest) -> String {
    nonce_summary::nonce_summary_bucket(item.weighted_value())
}
