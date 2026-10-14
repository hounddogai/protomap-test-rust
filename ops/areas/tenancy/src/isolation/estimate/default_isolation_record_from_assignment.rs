use super::*;

use std::collections::BTreeMap;

use crate::migration::bundle as migration_bundle;

pub fn summarize_migration_assignments_for_record(
    items: &[migration_bundle::MigrationAssignment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_default_isolation_records_from_assignment(
    items: &[migration_bundle::MigrationAssignment],
) -> Vec<DefaultIsolationRecord> {
    let mut out: Vec<DefaultIsolationRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_default_isolation_record(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_default_isolation_record(item: &DefaultIsolationRecord) -> String {
    migration_bundle::normalize_migration_bundle_code(&item.code)
}
