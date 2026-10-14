use super::*;

use std::collections::BTreeMap;

use crate::region_pin::manifest as region_pin_manifest;

pub fn summarize_region_pin_tiers_for_reservation(
    items: &[region_pin_manifest::RegionPinTier],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_region_pin_reservations_from_tier(
    items: &[region_pin_manifest::RegionPinTier],
) -> Vec<RegionPinReservation> {
    let mut out: Vec<RegionPinReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_region_pin_reservation(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_region_pin_reservation(item: &RegionPinReservation) -> f64 {
    region_pin_manifest::region_pin_tier_weight(2, true)
}
