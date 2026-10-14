use super::*;

use std::collections::BTreeMap;

use crate::plan::slot as plan_slot;

pub fn summarize_plan_policies_for_budget(items: &[plan_slot::PlanPolicy]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_migration_budgets_from_policy(items: &[plan_slot::PlanPolicy]) -> Vec<MigrationBudget> {
    let mut out: Vec<MigrationBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_migration_budget(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_migration_budget(item: &MigrationBudget) -> String {
    plan_slot::normalize_plan_review_code(&item.code)
}
