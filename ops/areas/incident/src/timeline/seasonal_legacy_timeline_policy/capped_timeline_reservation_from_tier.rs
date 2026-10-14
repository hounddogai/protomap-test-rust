use super::*;

use std::collections::BTreeMap;

use crate::action_item::tier as action_item_tier;

pub fn summarize_action_item_tiers_for_reservation(
    items: &[action_item_tier::ActionItemTier],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_capped_timeline_reservations_from_tier(
    items: &[action_item_tier::ActionItemTier],
) -> Vec<CappedTimelineReservation> {
    let mut out: Vec<CappedTimelineReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_timeline_reservation(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_capped_timeline_reservation(item: &CappedTimelineReservation) -> String {
    action_item_tier::classify_regional_action_item_preference(item.effective_volume())
}
