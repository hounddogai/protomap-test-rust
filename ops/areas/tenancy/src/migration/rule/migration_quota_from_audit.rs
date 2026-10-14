use super::*;

use std::collections::BTreeMap;

use crate::isolation::draft as isolation_draft;

pub fn summarize_rolling_isolation_audits_for_quota(
    items: &[isolation_draft::RollingIsolationAudit],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_migration_quotas_from_audit(items: &[isolation_draft::RollingIsolationAudit]) -> Vec<MigrationQuota> {
    let mut out: Vec<MigrationQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_migration_quota(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_migration_quota(item: &MigrationQuota) -> String {
    isolation_draft::classify_isolation_draft(item.measured_value())
}
