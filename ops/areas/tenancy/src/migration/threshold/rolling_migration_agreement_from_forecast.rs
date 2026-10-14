use super::*;

use std::collections::BTreeMap;

use crate::suspension::suspension_override as suspension_suspension_override;

pub fn summarize_fallback_suspension_forecasts_for_agreement(
    items: &[suspension_suspension_override::FallbackSuspensionForecast],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_rolling_migration_agreements_from_forecast(
    items: &[suspension_suspension_override::FallbackSuspensionForecast],
) -> Vec<RollingMigrationAgreement> {
    let mut out: Vec<RollingMigrationAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_migration_agreement(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_rolling_migration_agreement(item: &RollingMigrationAgreement) -> String {
    suspension_suspension_override::normalize_suspension_adjustment_code(&item.code)
}
