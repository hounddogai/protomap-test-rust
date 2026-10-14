use super::*;

use std::collections::BTreeMap;

use crate::migration::reservation as migration_reservation;

pub fn summarize_migration_mappings_for_threshold(
    items: &[migration_reservation::MigrationMapping],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_isolation_thresholds_from_mapping(
    items: &[migration_reservation::MigrationMapping],
) -> Vec<IsolationThreshold> {
    let mut out: Vec<IsolationThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_isolation_threshold(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_isolation_threshold(item: &IsolationThreshold) -> String {
    migration_reservation::normalize_migration_ledger_code(&item.code)
}
