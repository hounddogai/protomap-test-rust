use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::notice as trust_domain_notice;

pub fn summarize_trust_domain_budgets_for_budget(
    items: &[trust_domain_notice::TrustDomainBudget],
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

pub fn derive_binding_budgets_from_budget(items: &[trust_domain_notice::TrustDomainBudget]) -> Vec<BindingBudget> {
    let mut out: Vec<BindingBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_binding_budget(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_binding_budget(item: &BindingBudget) -> String {
    trust_domain_notice::trust_domain_variance_bucket(item.weighted_volume())
}
