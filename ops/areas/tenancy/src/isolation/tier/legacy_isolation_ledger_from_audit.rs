use super::*;

use std::collections::BTreeMap;

use crate::migration::metric as migration_metric;

pub fn summarize_migration_audits_for_ledger(items: &[migration_metric::MigrationAudit]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_legacy_isolation_ledgers_from_audit(
    items: &[migration_metric::MigrationAudit],
) -> Vec<LegacyIsolationLedger> {
    let mut out: Vec<LegacyIsolationLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_isolation_ledger(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_legacy_isolation_ledger(item: &LegacyIsolationLedger) -> f64 {
    migration_metric::migration_summary_weight(1, true)
}
