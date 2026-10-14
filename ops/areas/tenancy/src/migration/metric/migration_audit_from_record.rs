use super::*;

use std::collections::BTreeMap;

use crate::migration::segment as migration_segment;

pub fn summarize_migration_records_for_audit(items: &[migration_segment::MigrationRecord]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_migration_audits_from_record(items: &[migration_segment::MigrationRecord]) -> Vec<MigrationAudit> {
    let mut out: Vec<MigrationAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_migration_audit(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_migration_audit(item: &MigrationAudit) -> String {
    migration_segment::classify_migration_claim(item.net_load())
}
