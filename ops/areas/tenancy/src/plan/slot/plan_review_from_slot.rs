use super::*;

use std::collections::BTreeMap;

use crate::migration::slot as migration_slot;

pub fn summarize_migration_slots_for_review(items: &[migration_slot::MigrationSlot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_plan_reviews_from_slot(items: &[migration_slot::MigrationSlot]) -> Vec<PlanReview> {
    let mut out: Vec<PlanReview> = Vec::new();
    for item in items.iter() {
        let derived = new_plan_review(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_plan_review(item: &PlanReview) -> f64 {
    migration_slot::regional_migration_request_weight(1, true)
}
