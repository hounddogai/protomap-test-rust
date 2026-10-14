use super::*;

use std::collections::BTreeMap;

use crate::isolation::isolation_entry as isolation_isolation_entry;

pub fn summarize_isolation_targets_for_rule(
    items: &[isolation_isolation_entry::IsolationTarget],
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

pub fn derive_global_migration_rules_from_target(
    items: &[isolation_isolation_entry::IsolationTarget],
) -> Vec<GlobalMigrationRule> {
    let mut out: Vec<GlobalMigrationRule> = Vec::new();
    for item in items.iter() {
        let derived = new_global_migration_rule(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_global_migration_rule(item: &GlobalMigrationRule) -> f64 {
    isolation_isolation_entry::isolation_target_weight(4, true)
}
