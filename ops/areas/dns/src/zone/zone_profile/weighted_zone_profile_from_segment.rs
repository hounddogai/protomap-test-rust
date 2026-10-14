use super::*;

use std::collections::BTreeMap;

use crate::zone::audit as zone_audit;

pub fn summarize_fallback_zone_segments_for_profile(
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

pub fn derive_weighted_zone_profiles_from_segment(
    items: &[zone_audit::FallbackZoneSegment],
) -> Vec<WeightedZoneProfile> {
    let mut out: Vec<WeightedZoneProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_zone_profile(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_weighted_zone_profile(item: &WeightedZoneProfile) -> String {
    zone_audit::classify_scheduled_zone_reservation(item.effective_exposure())
}
