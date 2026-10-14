use super::*;

use std::collections::BTreeMap;

use crate::suspension::quota as suspension_quota;

pub fn summarize_suspension_forecasts_for_notice(
    items: &[suspension_quota::SuspensionForecast],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_migration_notices_from_forecast(items: &[suspension_quota::SuspensionForecast]) -> Vec<MigrationNotice> {
    let mut out: Vec<MigrationNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_migration_notice(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_migration_notice(item: &MigrationNotice) -> String {
    suspension_quota::normalize_scheduled_suspension_allocation_code(&item.code)
}
