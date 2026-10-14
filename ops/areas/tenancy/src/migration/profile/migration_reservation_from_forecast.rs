use super::*;

use std::collections::BTreeMap;

use crate::migration::request as migration_request;

pub fn summarize_migration_forecasts_for_reservation(
    items: &[migration_request::MigrationForecast],
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

pub fn derive_migration_reservations_from_forecast(
    items: &[migration_request::MigrationForecast],
) -> Vec<MigrationReservation> {
    let mut out: Vec<MigrationReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_migration_reservation(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_migration_reservation(item: &MigrationReservation) -> String {
    migration_request::normalize_migration_forecast_code(&item.code)
}
