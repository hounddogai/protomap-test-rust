use super::*;

use std::collections::BTreeMap;

use crate::migration::profile as migration_profile;

pub fn summarize_migration_reservations_for_adjustment(
    items: &[migration_profile::MigrationReservation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_plan_adjustments_from_reservation(
    items: &[migration_profile::MigrationReservation],
) -> Vec<PlanAdjustment> {
    let mut out: Vec<PlanAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_plan_adjustment(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_plan_adjustment(item: &PlanAdjustment) -> String {
    migration_profile::normalize_migration_profile_code(&item.code)
}
