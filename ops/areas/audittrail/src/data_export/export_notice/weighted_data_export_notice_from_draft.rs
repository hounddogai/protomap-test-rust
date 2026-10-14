use super::*;

use std::collections::BTreeMap;

use crate::sink::snapshot as sink_snapshot;

pub fn summarize_lenient_sink_drafts_for_notice(items: &[sink_snapshot::LenientSinkDraft]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_weighted_data_export_notices_from_draft(
    items: &[sink_snapshot::LenientSinkDraft],
) -> Vec<WeightedDataExportNotice> {
    let mut out: Vec<WeightedDataExportNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_data_export_notice(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_weighted_data_export_notice(item: &WeightedDataExportNotice) -> String {
    sink_snapshot::normalize_sink_reservation_code(&item.code)
}
