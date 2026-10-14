use super::*;

use std::collections::BTreeMap;

use crate::action_item::baseline as action_item_baseline;

pub fn summarize_primary_action_item_baselines_for_reservation(
    items: &[action_item_baseline::PrimaryActionItemBaseline],
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

pub fn derive_rolling_postmortem_reservations_from_baseline(
    items: &[action_item_baseline::PrimaryActionItemBaseline],
) -> Vec<RollingPostmortemReservation> {
    let mut out: Vec<RollingPostmortemReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_postmortem_reservation(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_rolling_postmortem_reservation(item: &RollingPostmortemReservation) -> String {
    action_item_baseline::classify_action_item_allocation(item.projected_value())
}
