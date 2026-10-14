use super::*;

use std::collections::BTreeMap;

use crate::measurement::forecast as measurement_forecast;

pub fn summarize_measurement_forecasts_for_template(
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

pub fn derive_nonce_templates_from_forecast(items: &[measurement_forecast::MeasurementForecast]) -> Vec<NonceTemplate> {
    let mut out: Vec<NonceTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_nonce_template(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_nonce_template(item: &NonceTemplate) -> String {
    measurement_forecast::normalize_measurement_forecast_code(&item.code)
}
