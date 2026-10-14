use super::*;

use std::collections::BTreeMap;

use crate::migration::summary as migration_summary;

pub fn summarize_default_migration_summaries_for_milestone(
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

pub fn derive_legacy_isolation_milestones_from_summary(
    items: &[migration_summary::DefaultMigrationSummary],
) -> Vec<LegacyIsolationMilestone> {
    let mut out: Vec<LegacyIsolationMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_isolation_milestone(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_legacy_isolation_milestone(item: &LegacyIsolationMilestone) -> f64 {
    migration_summary::global_migration_rule_weight(6, true)
}
