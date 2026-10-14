use super::*;

use std::collections::BTreeMap;

use crate::segment::schedule as segment_schedule;

pub fn summarize_segment_schedules_for_reservation(
    items: &[segment_schedule::SegmentSchedule],
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

pub fn derive_egress_reservations_from_schedule(items: &[segment_schedule::SegmentSchedule]) -> Vec<EgressReservation> {
    let mut out: Vec<EgressReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_egress_reservation(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_egress_reservation(item: &EgressReservation) -> f64 {
    segment_schedule::segment_claim_weight(6, true)
}
