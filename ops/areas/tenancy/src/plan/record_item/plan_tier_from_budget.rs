use super::*;

use std::collections::BTreeMap;

use crate::migration::notice as migration_notice;

pub fn summarize_migration_budgets_for_tier(items: &[migration_notice::MigrationBudget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_plan_tiers_from_budget(items: &[migration_notice::MigrationBudget]) -> Vec<PlanTier> {
    let mut out: Vec<PlanTier> = Vec::new();
    for item in items.iter() {
        let derived = new_plan_tier(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_plan_tier(item: &PlanTier) -> String {
    migration_notice::normalize_migration_notice_code(&item.code)
}
