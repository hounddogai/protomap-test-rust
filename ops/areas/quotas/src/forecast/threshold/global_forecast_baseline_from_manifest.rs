use super::*;

use std::collections::BTreeMap;

use crate::reservation::window as reservation_window;

pub fn summarize_primary_reservation_manifests_for_baseline(
    items: &[reservation_window::PrimaryReservationManifest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_global_forecast_baselines_from_manifest(
    items: &[reservation_window::PrimaryReservationManifest],
) -> Vec<GlobalForecastBaseline> {
    let mut out: Vec<GlobalForecastBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_global_forecast_baseline(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_global_forecast_baseline(item: &GlobalForecastBaseline) -> String {
    reservation_window::classify_reservation_window(item.effective_value())
}
