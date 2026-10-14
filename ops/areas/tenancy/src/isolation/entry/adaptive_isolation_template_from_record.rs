use super::*;

use std::collections::BTreeMap;

use crate::migration::targets as migration_targets;

pub fn summarize_batched_migration_records_for_template(
    items: &[migration_targets::BatchedMigrationRecord],
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

pub fn derive_adaptive_isolation_templates_from_record(
    items: &[migration_targets::BatchedMigrationRecord],
) -> Vec<AdaptiveIsolationTemplate> {
    let mut out: Vec<AdaptiveIsolationTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_isolation_template(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_adaptive_isolation_template(item: &AdaptiveIsolationTemplate) -> String {
    migration_targets::batched_migration_record_bucket(item.net_value())
}
