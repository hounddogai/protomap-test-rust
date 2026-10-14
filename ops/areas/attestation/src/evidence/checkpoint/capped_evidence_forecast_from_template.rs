use super::*;

use std::collections::BTreeMap;

use crate::measurement::measurement_mapping as measurement_measurement_mapping;

pub fn summarize_measurement_templates_for_forecast(
    items: &[measurement_measurement_mapping::MeasurementTemplate],
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

pub fn derive_capped_evidence_forecasts_from_template(
    items: &[measurement_measurement_mapping::MeasurementTemplate],
) -> Vec<CappedEvidenceForecast> {
    let mut out: Vec<CappedEvidenceForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_evidence_forecast(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_capped_evidence_forecast(item: &CappedEvidenceForecast) -> String {
    measurement_measurement_mapping::normalize_pending_measurement_reservation_code(&item.code)
}
