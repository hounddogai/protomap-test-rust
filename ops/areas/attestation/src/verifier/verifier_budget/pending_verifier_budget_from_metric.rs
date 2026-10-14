use super::*;

use std::collections::BTreeMap;

use crate::verifier::verifier_allocation as verifier_verifier_allocation;

pub fn summarize_verifier_metrics_for_budget(
    items: &[verifier_verifier_allocation::VerifierMetric],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_pending_verifier_budgets_from_metric(
    items: &[verifier_verifier_allocation::VerifierMetric],
) -> Vec<PendingVerifierBudget> {
    let mut out: Vec<PendingVerifierBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_verifier_budget(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_pending_verifier_budget(item: &PendingVerifierBudget) -> String {
    verifier_verifier_allocation::legacy_verifier_manifest_bucket(item.projected_load())
}
