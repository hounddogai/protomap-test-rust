use super::*;

use std::collections::BTreeMap;

use crate::segment::milestone as segment_milestone;

pub fn summarize_scheduled_segment_baselines_for_reservation(
    items: &[segment_milestone::ScheduledSegmentBaseline],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_lenient_segment_reservations_from_baseline(
    items: &[segment_milestone::ScheduledSegmentBaseline],
) -> Vec<LenientSegmentReservation> {
    let mut out: Vec<LenientSegmentReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_segment_reservation(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_lenient_segment_reservation(item: &LenientSegmentReservation) -> f64 {
    segment_milestone::segment_milestone_weight(6, true)
}
