use super::*;

use std::collections::BTreeMap;

use crate::forecast::forecast_policy as forecast_forecast_policy;

pub fn summarize_batched_forecast_manifests_for_reservation(
    items: &[forecast_forecast_policy::BatchedForecastManifest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_capped_overage_reservations_from_manifest(
    items: &[forecast_forecast_policy::BatchedForecastManifest],
) -> Vec<CappedOverageReservation> {
    let mut out: Vec<CappedOverageReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_overage_reservation(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_capped_overage_reservation(item: &CappedOverageReservation) -> String {
    forecast_forecast_policy::normalize_default_forecast_policy_code(&item.code)
}
