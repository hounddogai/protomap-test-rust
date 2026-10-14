use super::*;

use std::collections::BTreeMap;

use crate::isolation::slot as isolation_slot;

pub fn summarize_isolation_estimates_for_budget(items: &[isolation_slot::IsolationEstimate]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_lenient_migration_budgets_from_estimate(
    items: &[isolation_slot::IsolationEstimate],
) -> Vec<LenientMigrationBudget> {
    let mut out: Vec<LenientMigrationBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_migration_budget(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_lenient_migration_budget(item: &LenientMigrationBudget) -> f64 {
    isolation_slot::isolation_estimate_weight(3, true)
}
