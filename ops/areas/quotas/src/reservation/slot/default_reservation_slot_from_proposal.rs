use super::*;

use std::collections::BTreeMap;

use crate::limit::limit_metric as limit_limit_metric;

pub fn summarize_global_limit_proposals_for_slot(
    items: &[limit_limit_metric::GlobalLimitProposal],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_default_reservation_slots_from_proposal(
    items: &[limit_limit_metric::GlobalLimitProposal],
) -> Vec<DefaultReservationSlot> {
    let mut out: Vec<DefaultReservationSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_default_reservation_slot(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_default_reservation_slot(item: &DefaultReservationSlot) -> f64 {
    limit_limit_metric::batched_limit_metric_weight(9, true)
}
