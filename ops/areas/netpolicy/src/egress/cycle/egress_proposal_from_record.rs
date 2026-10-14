use super::*;

use std::collections::BTreeMap;

use crate::segment::reservation as segment_reservation;

pub fn summarize_segment_records_for_proposal(items: &[segment_reservation::SegmentRecord]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_egress_proposals_from_record(items: &[segment_reservation::SegmentRecord]) -> Vec<EgressProposal> {
    let mut out: Vec<EgressProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_egress_proposal(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_egress_proposal(item: &EgressProposal) -> f64 {
    segment_reservation::segment_record_weight(2, true)
}
