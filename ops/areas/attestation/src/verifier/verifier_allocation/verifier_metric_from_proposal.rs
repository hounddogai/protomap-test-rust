use super::*;

use std::collections::BTreeMap;

use crate::nonce::rule as nonce_rule;

pub fn summarize_manual_nonce_proposals_for_metric(items: &[nonce_rule::ManualNonceProposal]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_verifier_metrics_from_proposal(items: &[nonce_rule::ManualNonceProposal]) -> Vec<VerifierMetric> {
    let mut out: Vec<VerifierMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_verifier_metric(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_verifier_metric(item: &VerifierMetric) -> f64 {
    nonce_rule::adaptive_nonce_rule_weight(5, true)
}
