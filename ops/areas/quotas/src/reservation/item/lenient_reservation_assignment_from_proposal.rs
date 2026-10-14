use super::*;

use std::collections::BTreeMap;

use crate::usage::proposal as usage_proposal;

pub fn summarize_tiered_usage_proposals_for_assignment(
    items: &[usage_proposal::TieredUsageProposal],
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

pub fn derive_lenient_reservation_assignments_from_proposal(
    items: &[usage_proposal::TieredUsageProposal],
) -> Vec<LenientReservationAssignment> {
    let mut out: Vec<LenientReservationAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_reservation_assignment(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_lenient_reservation_assignment(item: &LenientReservationAssignment) -> String {
    usage_proposal::normalize_usage_slot_code(&item.code)
}
