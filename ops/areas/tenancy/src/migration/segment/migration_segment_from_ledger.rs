use super::*;

use std::collections::BTreeMap;

use crate::isolation::ledger as isolation_ledger;

pub fn summarize_isolation_ledgers_for_segment(items: &[isolation_ledger::IsolationLedger]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_migration_segments_from_ledger(items: &[isolation_ledger::IsolationLedger]) -> Vec<MigrationSegment> {
    let mut out: Vec<MigrationSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_migration_segment(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_migration_segment(item: &MigrationSegment) -> String {
    isolation_ledger::classify_isolation_milestone(item.adjusted_value())
}
