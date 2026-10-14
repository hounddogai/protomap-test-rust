use super::*;

use std::collections::BTreeMap;

use crate::zone::zone_digest as zone_zone_digest;

pub fn summarize_global_zone_digests_for_reservation(
    items: &[zone_zone_digest::GlobalZoneDigest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_tiered_health_check_reservations_from_digest(
    items: &[zone_zone_digest::GlobalZoneDigest],
) -> Vec<TieredHealthCheckReservation> {
    let mut out: Vec<TieredHealthCheckReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_health_check_reservation(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_tiered_health_check_reservation(item: &TieredHealthCheckReservation) -> String {
    zone_zone_digest::classify_global_zone_digest(item.measured_value())
}
