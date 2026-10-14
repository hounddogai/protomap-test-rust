use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::scheduled_burn_cycle as budget_burn_scheduled_burn_cycle;

pub fn summarize_budget_burn_milestones_for_bundle(
    items: &[budget_burn_scheduled_burn_cycle::BudgetBurnMilestone],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_budget_burn_bundles_from_milestone(
    items: &[budget_burn_scheduled_burn_cycle::BudgetBurnMilestone],
) -> Vec<BudgetBurnBundle> {
    let mut out: Vec<BudgetBurnBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_budget_burn_bundle(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_budget_burn_bundle(item: &BudgetBurnBundle) -> String {
    budget_burn_scheduled_burn_cycle::normalize_budget_burn_digest_code(&item.code)
}
