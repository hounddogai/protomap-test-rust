use super::*;

use std::collections::BTreeMap;

use crate::peering::record_item as peering_record_item;

pub fn summarize_peering_allocations_for_adjustment(
    items: &[peering_record_item::PeeringAllocation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_tiered_segment_adjustments_from_allocation(
    items: &[peering_record_item::PeeringAllocation],
) -> Vec<TieredSegmentAdjustment> {
    let mut out: Vec<TieredSegmentAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_segment_adjustment(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_tiered_segment_adjustment(item: &TieredSegmentAdjustment) -> String {
    peering_record_item::classify_capped_peering_record(item.net_load())
}
