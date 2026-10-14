use super::*;

use std::collections::BTreeMap;

use crate::isolation::entry as isolation_entry;

pub fn summarize_isolation_entries_for_score(items: &[isolation_entry::IsolationEntry]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_migration_scores_from_entry(items: &[isolation_entry::IsolationEntry]) -> Vec<MigrationScore> {
    let mut out: Vec<MigrationScore> = Vec::new();
    for item in items.iter() {
        let derived = new_migration_score(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_migration_score(item: &MigrationScore) -> String {
    isolation_entry::normalize_isolation_entry_code(&item.code)
}
