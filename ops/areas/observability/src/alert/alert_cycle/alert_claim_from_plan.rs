use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::allocation as budget_burn_allocation;

pub fn summarize_budget_burn_plans_for_claim(
    items: &[budget_burn_allocation::BudgetBurnPlan],
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

pub fn derive_alert_claims_from_plan(items: &[budget_burn_allocation::BudgetBurnPlan]) -> Vec<AlertClaim> {
    let mut out: Vec<AlertClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_alert_claim(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_alert_claim(item: &AlertClaim) -> String {
    budget_burn_allocation::normalize_budget_burn_allocation_code(&item.code)
}
