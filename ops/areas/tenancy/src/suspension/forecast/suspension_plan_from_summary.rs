use super::*;

use std::collections::BTreeMap;

use crate::migration::summary as migration_summary;

pub fn summarize_default_migration_summaries_for_plan(
    items: &[migration_summary::DefaultMigrationSummary],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_suspension_plans_from_summary(
    items: &[migration_summary::DefaultMigrationSummary],
) -> Vec<SuspensionPlan> {
    let mut out: Vec<SuspensionPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_suspension_plan(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_suspension_plan(item: &SuspensionPlan) -> String {
    migration_summary::global_migration_rule_bucket(item.measured_value())
}
