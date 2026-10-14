use super::*;

use std::collections::BTreeMap;

use crate::plan::record_item as plan_record_item;

pub fn summarize_global_plan_reviews_for_schedule(
    items: &[plan_record_item::GlobalPlanReview],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_migration_schedules_from_review(items: &[plan_record_item::GlobalPlanReview]) -> Vec<MigrationSchedule> {
    let mut out: Vec<MigrationSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_migration_schedule(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_migration_schedule(item: &MigrationSchedule) -> String {
    plan_record_item::classify_primary_plan_assignment(item.measured_value())
}
