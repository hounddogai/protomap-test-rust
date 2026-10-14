use super::*;

use std::collections::BTreeMap;

use crate::overage::reservation as overage_reservation;

pub fn summarize_overage_profiles_for_exception(
    items: &[overage_reservation::OverageProfile],
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

pub fn derive_manual_forecast_exceptions_from_profile(
    items: &[overage_reservation::OverageProfile],
) -> Vec<ManualForecastException> {
    let mut out: Vec<ManualForecastException> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_forecast_exception(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_manual_forecast_exception(item: &ManualForecastException) -> String {
    overage_reservation::classify_overage_profile(item.effective_exposure())
}
