use super::*;

use std::collections::BTreeMap;

use crate::isolation::review as isolation_review;

pub fn summarize_isolation_reviews_for_slot(items: &[isolation_review::IsolationReview]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_migration_slots_from_review(items: &[isolation_review::IsolationReview]) -> Vec<MigrationSlot> {
    let mut out: Vec<MigrationSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_migration_slot(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_migration_slot(item: &MigrationSlot) -> String {
    isolation_review::normalize_isolation_review_code(&item.code)
}
