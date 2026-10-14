use super::*;

use std::collections::BTreeMap;

use crate::segment::entry as segment_entry;

pub fn summarize_segment_groups_for_slot(items: &[segment_entry::SegmentGroup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_peering_slots_from_group(items: &[segment_entry::SegmentGroup]) -> Vec<PeeringSlot> {
    let mut out: Vec<PeeringSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_peering_slot(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_peering_slot(item: &PeeringSlot) -> f64 {
    segment_entry::fallback_segment_entry_weight(1, true)
}
