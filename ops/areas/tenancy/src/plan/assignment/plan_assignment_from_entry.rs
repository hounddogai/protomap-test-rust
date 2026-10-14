use super::*;

use std::collections::BTreeMap;

use crate::migration::rollup as migration_rollup;

pub fn summarize_migration_entries_for_assignment(items: &[migration_rollup::MigrationEntry]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_plan_assignments_from_entry(items: &[migration_rollup::MigrationEntry]) -> Vec<PlanAssignment> {
    let mut out: Vec<PlanAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_plan_assignment(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_plan_assignment(item: &PlanAssignment) -> f64 {
    migration_rollup::migration_entry_weight(2, true)
}
