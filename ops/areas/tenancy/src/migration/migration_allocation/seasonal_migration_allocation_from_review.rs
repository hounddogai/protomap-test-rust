use super::*;

use std::collections::BTreeMap;

use crate::migration::migration_segment as migration_migration_segment;

pub fn summarize_migration_reviews_for_allocation(
    items: &[migration_migration_segment::MigrationReview],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_seasonal_migration_allocations_from_review(
    items: &[migration_migration_segment::MigrationReview],
) -> Vec<SeasonalMigrationAllocation> {
    let mut out: Vec<SeasonalMigrationAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_migration_allocation(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_seasonal_migration_allocation(item: &SeasonalMigrationAllocation) -> String {
    migration_migration_segment::scheduled_migration_segment_bucket(item.measured_value())
}
