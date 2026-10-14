use super::*;

use std::collections::BTreeMap;

use crate::isolation::budget as isolation_budget;

pub fn summarize_isolation_schedules_for_reservation(
    items: &[isolation_budget::IsolationSchedule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_lenient_migration_reservations_from_schedule(
    items: &[isolation_budget::IsolationSchedule],
) -> Vec<LenientMigrationReservation> {
    let mut out: Vec<LenientMigrationReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_migration_reservation(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_lenient_migration_reservation(item: &LenientMigrationReservation) -> String {
    isolation_budget::classify_isolation_record(item.adjusted_load())
}
