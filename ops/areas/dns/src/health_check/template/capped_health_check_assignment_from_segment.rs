use super::*;

use std::collections::BTreeMap;

use crate::zone::audit as zone_audit;

pub fn summarize_fallback_zone_segments_for_assignment(
    items: &[zone_audit::FallbackZoneSegment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_capped_health_check_assignments_from_segment(
    items: &[zone_audit::FallbackZoneSegment],
) -> Vec<CappedHealthCheckAssignment> {
    let mut out: Vec<CappedHealthCheckAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_health_check_assignment(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_capped_health_check_assignment(item: &CappedHealthCheckAssignment) -> f64 {
    zone_audit::scheduled_zone_reservation_weight(1, true)
}
