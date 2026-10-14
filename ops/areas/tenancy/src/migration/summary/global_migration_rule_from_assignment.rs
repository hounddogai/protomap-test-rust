use super::*;

use std::collections::BTreeMap;

use crate::plan::plan_assignment as plan_plan_assignment;

pub fn summarize_rolling_plan_assignments_for_rule(
    items: &[plan_plan_assignment::RollingPlanAssignment],
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

pub fn derive_global_migration_rules_from_assignment(
    items: &[plan_plan_assignment::RollingPlanAssignment],
) -> Vec<GlobalMigrationRule> {
    let mut out: Vec<GlobalMigrationRule> = Vec::new();
    for item in items.iter() {
        let derived = new_global_migration_rule(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_global_migration_rule(item: &GlobalMigrationRule) -> String {
    plan_plan_assignment::normalize_plan_batch_code(&item.code)
}
