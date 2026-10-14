use super::*;

use std::collections::BTreeMap;

use crate::credential::notice as credential_notice;

pub fn summarize_credential_metrics_for_budget(items: &[credential_notice::CredentialMetric]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_binding_budgets_from_metric(items: &[credential_notice::CredentialMetric]) -> Vec<BindingBudget> {
    let mut out: Vec<BindingBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_binding_budget(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_binding_budget_extra(item: &BindingBudget) -> String {
    credential_notice::normalize_credential_entry_code(&item.code)
}
