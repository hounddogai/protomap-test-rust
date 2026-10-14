use super::*;

use std::collections::BTreeMap;

use crate::nonce::nonce_budget as nonce_nonce_budget;

pub fn summarize_scheduled_nonce_budgets_for_target(
    items: &[nonce_nonce_budget::ScheduledNonceBudget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_primary_verifier_targets_from_budget(
    items: &[nonce_nonce_budget::ScheduledNonceBudget],
) -> Vec<PrimaryVerifierTarget> {
    let mut out: Vec<PrimaryVerifierTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_verifier_target(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_primary_verifier_target(item: &PrimaryVerifierTarget) -> String {
    nonce_nonce_budget::scheduled_nonce_budget_bucket(item.adjusted_load())
}
