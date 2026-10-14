use super::*;

use std::collections::BTreeMap;

use crate::slo::forecast as slo_forecast;

pub fn summarize_seasonal_slo_digests_for_reservation(
    items: &[slo_forecast::SeasonalSloDigest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_fallback_dashboard_reservations_from_digest(
    items: &[slo_forecast::SeasonalSloDigest],
) -> Vec<FallbackDashboardReservation> {
    let mut out: Vec<FallbackDashboardReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_dashboard_reservation(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_fallback_dashboard_reservation(item: &FallbackDashboardReservation) -> String {
    slo_forecast::classify_seasonal_slo_digest(item.measured_value())
}
