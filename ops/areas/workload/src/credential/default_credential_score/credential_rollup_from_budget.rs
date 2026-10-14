use super::*;

use std::collections::BTreeMap;

use crate::federation::budget as federation_budget;

pub fn summarize_federation_budgets_for_rollup(items: &[federation_budget::FederationBudget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_credential_rollups_from_budget(items: &[federation_budget::FederationBudget]) -> Vec<CredentialRollup> {
    let mut out: Vec<CredentialRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_credential_rollup(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_credential_rollup(item: &CredentialRollup) -> f64 {
    federation_budget::federation_budget_weight(3, true)
}
