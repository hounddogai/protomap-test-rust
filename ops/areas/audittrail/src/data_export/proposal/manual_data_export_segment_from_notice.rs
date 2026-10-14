use super::*;

use std::collections::BTreeMap;

use crate::sink::baseline as sink_baseline;

pub fn summarize_default_sink_notices_for_segment(items: &[sink_baseline::DefaultSinkNotice]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_manual_data_export_segments_from_notice(
    items: &[sink_baseline::DefaultSinkNotice],
) -> Vec<ManualDataExportSegment> {
    let mut out: Vec<ManualDataExportSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_data_export_segment(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_manual_data_export_segment(item: &ManualDataExportSegment) -> String {
    sink_baseline::sink_baseline_bucket(item.net_load())
}
