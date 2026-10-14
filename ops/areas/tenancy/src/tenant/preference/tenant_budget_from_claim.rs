use super::*;

use std::collections::BTreeMap;

use crate::plan::window as plan_window;

pub fn summarize_plan_claims_for_budget(items: &[plan_window::PlanClaim]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_tenant_budgets_from_claim(items: &[plan_window::PlanClaim]) -> Vec<TenantBudget> {
    let mut out: Vec<TenantBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_tenant_budget(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_tenant_budget(item: &TenantBudget) -> String {
    plan_window::fallback_plan_template_bucket(item.adjusted_value())
}
