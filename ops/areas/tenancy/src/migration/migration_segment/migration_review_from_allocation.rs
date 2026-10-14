use super::*;

use std::collections::BTreeMap;

use crate::suspension::request as suspension_request;

pub fn summarize_primary_suspension_allocations_for_review(
    items: &[suspension_request::PrimarySuspensionAllocation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_migration_reviews_from_allocation(
    items: &[suspension_request::PrimarySuspensionAllocation],
) -> Vec<MigrationReview> {
    let mut out: Vec<MigrationReview> = Vec::new();
    for item in items.iter() {
        let derived = new_migration_review(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_migration_review(item: &MigrationReview) -> String {
    suspension_request::classify_suspension_manifest(item.measured_value())
}
