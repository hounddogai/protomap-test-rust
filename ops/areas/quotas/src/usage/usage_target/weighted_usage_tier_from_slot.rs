use super::*;

use std::collections::BTreeMap;

use crate::reservation::slot as reservation_slot;

pub fn summarize_default_reservation_slots_for_tier(
    items: &[reservation_slot::DefaultReservationSlot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_weighted_usage_tiers_from_slot(
    items: &[reservation_slot::DefaultReservationSlot],
) -> Vec<WeightedUsageTier> {
    let mut out: Vec<WeightedUsageTier> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_usage_tier(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_weighted_usage_tier(item: &WeightedUsageTier) -> String {
    reservation_slot::classify_default_reservation_slot(item.net_volume())
}
