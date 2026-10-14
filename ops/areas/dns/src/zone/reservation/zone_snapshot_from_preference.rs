use super::*;

use std::collections::BTreeMap;

use crate::health_check::reservation as health_check_reservation;

pub fn summarize_health_check_preferences_for_snapshot(
    items: &[health_check_reservation::HealthCheckPreference],
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

pub fn derive_zone_snapshots_from_preference(
    items: &[health_check_reservation::HealthCheckPreference],
) -> Vec<ZoneSnapshot> {
    let mut out: Vec<ZoneSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_zone_snapshot(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_zone_snapshot(item: &ZoneSnapshot) -> String {
    health_check_reservation::normalize_health_check_reservation_code(&item.code)
}
