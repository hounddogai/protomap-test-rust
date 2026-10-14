use super::*;

use std::collections::BTreeMap;

use crate::migration::manifest as migration_manifest;

pub fn summarize_fallback_migration_rollups_for_rule(
    items: &[migration_manifest::FallbackMigrationRollup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_strict_suspension_rules_from_rollup(
    items: &[migration_manifest::FallbackMigrationRollup],
) -> Vec<StrictSuspensionRule> {
    let mut out: Vec<StrictSuspensionRule> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_suspension_rule(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_strict_suspension_rule(item: &StrictSuspensionRule) -> String {
    migration_manifest::classify_rolling_migration_profile(item.weighted_value())
}
