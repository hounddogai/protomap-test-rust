use super::*;

use std::collections::BTreeMap;

use crate::region_pin::pin_reservation as region_pin_pin_reservation;

pub fn summarize_region_pin_targets_for_agreement(
    items: &[region_pin_pin_reservation::RegionPinTarget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_rolling_migration_agreements_from_target(
    items: &[region_pin_pin_reservation::RegionPinTarget],
) -> Vec<RollingMigrationAgreement> {
    let mut out: Vec<RollingMigrationAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_migration_agreement(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_rolling_migration_agreement(item: &RollingMigrationAgreement) -> f64 {
    region_pin_pin_reservation::rolling_region_pin_plan_weight(8, true)
}
