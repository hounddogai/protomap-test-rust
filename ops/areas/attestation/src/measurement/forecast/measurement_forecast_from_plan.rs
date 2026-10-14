use super::*;

use std::collections::BTreeMap;

use crate::evidence::schedule as evidence_schedule;

pub fn summarize_evidence_plans_for_forecast(items: &[evidence_schedule::EvidencePlan]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_measurement_forecasts_from_plan(items: &[evidence_schedule::EvidencePlan]) -> Vec<MeasurementForecast> {
    let mut out: Vec<MeasurementForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_measurement_forecast(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_measurement_forecast(item: &MeasurementForecast) -> String {
    evidence_schedule::fallback_evidence_reservation_bucket(item.effective_value())
}
