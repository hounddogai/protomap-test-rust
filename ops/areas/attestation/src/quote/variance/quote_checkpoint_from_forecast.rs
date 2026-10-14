use super::*;

use std::collections::BTreeMap;

use crate::measurement::forecast as measurement_forecast;

pub fn summarize_measurement_forecasts_for_checkpoint(
    items: &[measurement_forecast::MeasurementForecast],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_quote_checkpoints_from_forecast(
    items: &[measurement_forecast::MeasurementForecast],
) -> Vec<QuoteCheckpoint> {
    let mut out: Vec<QuoteCheckpoint> = Vec::new();
    for item in items.iter() {
        let derived = new_quote_checkpoint(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_quote_checkpoint(item: &QuoteCheckpoint) -> String {
    measurement_forecast::normalize_measurement_forecast_code(&item.code)
}
