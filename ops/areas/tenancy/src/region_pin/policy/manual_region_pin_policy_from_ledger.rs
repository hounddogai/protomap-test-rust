use super::*;

use std::collections::BTreeMap;

use crate::migration::reservation as migration_reservation;

pub fn summarize_migration_ledgers_for_policy(
    items: &[migration_reservation::MigrationLedger],
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

pub fn derive_manual_region_pin_policies_from_ledger(
    items: &[migration_reservation::MigrationLedger],
) -> Vec<ManualRegionPinPolicy> {
    let mut out: Vec<ManualRegionPinPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_region_pin_policy(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_manual_region_pin_policy_extra(item: &ManualRegionPinPolicy) -> String {
    migration_reservation::normalize_lenient_migration_reservation_code(&item.code)
}
