use super::*;

use std::collections::BTreeMap;

use crate::migration::budget as migration_budget;

pub fn summarize_migration_policies_for_ledger(items: &[migration_budget::MigrationPolicy]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_pending_region_pin_ledgers_from_policy(
    items: &[migration_budget::MigrationPolicy],
) -> Vec<PendingRegionPinLedger> {
    let mut out: Vec<PendingRegionPinLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_region_pin_ledger(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_pending_region_pin_ledger(item: &PendingRegionPinLedger) -> f64 {
    migration_budget::capped_migration_forecast_weight(4, true)
}
