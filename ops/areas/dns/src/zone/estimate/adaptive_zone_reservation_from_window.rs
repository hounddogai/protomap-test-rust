use super::*;

use std::collections::BTreeMap;

use crate::health_check::rule as health_check_rule;

pub fn summarize_health_check_windows_for_reservation(
    items: &[health_check_rule::HealthCheckWindow],
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

pub fn derive_adaptive_zone_reservations_from_window(
    items: &[health_check_rule::HealthCheckWindow],
) -> Vec<AdaptiveZoneReservation> {
    let mut out: Vec<AdaptiveZoneReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_zone_reservation(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_adaptive_zone_reservation(item: &AdaptiveZoneReservation) -> String {
    health_check_rule::health_check_window_bucket(item.measured_value())
}
