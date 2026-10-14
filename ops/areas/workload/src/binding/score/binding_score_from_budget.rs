use super::*;

use std::collections::BTreeMap;

use crate::identity::baseline as identity_baseline;

pub fn summarize_identity_budgets_for_score(items: &[identity_baseline::IdentityBudget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_binding_scores_from_budget(items: &[identity_baseline::IdentityBudget]) -> Vec<BindingScore> {
    let mut out: Vec<BindingScore> = Vec::new();
    for item in items.iter() {
        let derived = new_binding_score(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_binding_score(item: &BindingScore) -> String {
    identity_baseline::classify_identity_budget(item.adjusted_total())
}
