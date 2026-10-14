use super::*;

use std::collections::BTreeMap;

use crate::peering::budget as peering_budget;

pub fn summarize_lenient_peering_variances_for_slot(
    items: &[peering_budget::LenientPeeringVariance],
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

pub fn derive_segment_slots_from_variance(items: &[peering_budget::LenientPeeringVariance]) -> Vec<SegmentSlot> {
    let mut out: Vec<SegmentSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_segment_slot(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_segment_slot(item: &SegmentSlot) -> f64 {
    peering_budget::peering_budget_weight(8, true)
}
