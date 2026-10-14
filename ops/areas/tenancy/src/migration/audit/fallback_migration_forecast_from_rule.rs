use super::*;

use std::collections::BTreeMap;

use crate::region_pin::rule as region_pin_rule;

pub fn summarize_adaptive_region_pin_rules_for_forecast(
    items: &[region_pin_rule::AdaptiveRegionPinRule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_fallback_migration_forecasts_from_rule(
    items: &[region_pin_rule::AdaptiveRegionPinRule],
) -> Vec<FallbackMigrationForecast> {
    let mut out: Vec<FallbackMigrationForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_migration_forecast(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_fallback_migration_forecast(item: &FallbackMigrationForecast) -> f64 {
    region_pin_rule::adaptive_region_pin_rule_weight(3, true)
}
