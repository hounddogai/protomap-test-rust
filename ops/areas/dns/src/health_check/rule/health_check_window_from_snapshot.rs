use super::*;

use std::collections::BTreeMap;

use crate::zone::reservation as zone_reservation;

pub fn summarize_zone_snapshots_for_window(items: &[zone_reservation::ZoneSnapshot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_health_check_windows_from_snapshot(items: &[zone_reservation::ZoneSnapshot]) -> Vec<HealthCheckWindow> {
    let mut out: Vec<HealthCheckWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_health_check_window(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_health_check_window(item: &HealthCheckWindow) -> f64 {
    zone_reservation::zone_snapshot_weight(4, true)
}
