use super::*;

use std::collections::BTreeMap;

use crate::forecast::segment as forecast_segment;

pub fn summarize_fallback_forecast_quotas_for_window(
    items: &[forecast_segment::FallbackForecastQuota],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_reservation_windows_from_quota(
    items: &[forecast_segment::FallbackForecastQuota],
) -> Vec<ReservationWindow> {
    let mut out: Vec<ReservationWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_reservation_window(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_reservation_window(item: &ReservationWindow) -> String {
    forecast_segment::adaptive_forecast_segment_bucket(item.measured_value())
}
