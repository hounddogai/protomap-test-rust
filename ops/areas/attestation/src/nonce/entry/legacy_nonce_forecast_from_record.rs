use super::*;

use std::collections::BTreeMap;

use crate::measurement::record_item as measurement_record_item;

pub fn summarize_tiered_measurement_records_for_forecast(
    items: &[measurement_record_item::TieredMeasurementRecord],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_legacy_nonce_forecasts_from_record(
    items: &[measurement_record_item::TieredMeasurementRecord],
) -> Vec<LegacyNonceForecast> {
    let mut out: Vec<LegacyNonceForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_nonce_forecast(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_legacy_nonce_forecast(item: &LegacyNonceForecast) -> String {
    measurement_record_item::classify_fallback_measurement_estimate(item.projected_value())
}
