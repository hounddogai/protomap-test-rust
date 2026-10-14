use super::*;

use std::collections::BTreeMap;

use crate::segment::record_item as segment_record_item;

pub fn summarize_capped_segment_records_for_agreement(
    items: &[segment_record_item::CappedSegmentRecord],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_peering_agreements_from_record(
    items: &[segment_record_item::CappedSegmentRecord],
) -> Vec<PeeringAgreement> {
    let mut out: Vec<PeeringAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_peering_agreement(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_peering_agreement(item: &PeeringAgreement) -> f64 {
    segment_record_item::primary_segment_group_weight(3, true)
}
