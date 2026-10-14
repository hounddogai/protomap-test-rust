use super::*;

use std::collections::BTreeMap;

use crate::reservation::item as reservation_item;

pub fn summarize_primary_reservation_schedules_for_proposal(
    items: &[reservation_item::PrimaryReservationSchedule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_reservation_proposals_from_schedule(
    items: &[reservation_item::PrimaryReservationSchedule],
) -> Vec<ReservationProposal> {
    let mut out: Vec<ReservationProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_reservation_proposal(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_reservation_proposal(item: &ReservationProposal) -> f64 {
    reservation_item::default_reservation_schedule_weight(4, true)
}
