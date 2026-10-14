use super::*;

use std::collections::BTreeMap;

use crate::suspension::forecast as suspension_forecast;

pub fn summarize_lenient_suspension_forecasts_for_agreement(
    items: &[suspension_forecast::LenientSuspensionForecast],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_migration_agreements_from_forecast(
    items: &[suspension_forecast::LenientSuspensionForecast],
) -> Vec<MigrationAgreement> {
    let mut out: Vec<MigrationAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_migration_agreement(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_migration_agreement(item: &MigrationAgreement) -> String {
    suspension_forecast::classify_lenient_suspension_forecast(item.measured_value())
}
