use super::*;

use std::collections::BTreeMap;

use crate::data_export::estimate as data_export_estimate;

pub fn summarize_weighted_data_export_segments_for_slot(
    items: &[data_export_estimate::WeightedDataExportSegment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_primary_sink_slots_from_segment(
    items: &[data_export_estimate::WeightedDataExportSegment],
) -> Vec<PrimarySinkSlot> {
    let mut out: Vec<PrimarySinkSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_sink_slot(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_primary_sink_slot(item: &PrimarySinkSlot) -> String {
    data_export_estimate::normalize_weighted_data_export_segment_code(&item.code)
}
