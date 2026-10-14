use super::*;

use std::collections::BTreeMap;

use crate::binding::adjustment as binding_adjustment;

pub fn summarize_binding_adjustments_for_budget(
    items: &[binding_adjustment::BindingAdjustment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_scheduled_trust_domain_budgets_from_adjustment(
    items: &[binding_adjustment::BindingAdjustment],
) -> Vec<ScheduledTrustDomainBudget> {
    let mut out: Vec<ScheduledTrustDomainBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_trust_domain_budget(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_scheduled_trust_domain_budget(item: &ScheduledTrustDomainBudget) -> String {
    binding_adjustment::classify_lenient_binding_preference(item.projected_total())
}
