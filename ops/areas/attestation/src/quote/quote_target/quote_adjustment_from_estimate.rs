use super::*;

use std::collections::BTreeMap;

use crate::measurement::window as measurement_window;

pub fn summarize_measurement_estimates_for_adjustment(
    items: &[measurement_window::MeasurementEstimate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_quote_adjustments_from_estimate(
    items: &[measurement_window::MeasurementEstimate],
) -> Vec<QuoteAdjustment> {
    let mut out: Vec<QuoteAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_quote_adjustment(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_quote_adjustment(item: &QuoteAdjustment) -> String {
    measurement_window::adaptive_measurement_entry_bucket(item.measured_value())
}
