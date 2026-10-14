use super::*;

use std::collections::BTreeMap;

use crate::redaction::redaction_group as redaction_redaction_group;

pub fn summarize_redaction_manifests_for_reservation(
    items: &[redaction_redaction_group::RedactionManifest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_data_export_reservations_from_manifest(
    items: &[redaction_redaction_group::RedactionManifest],
) -> Vec<DataExportReservation> {
    let mut out: Vec<DataExportReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_data_export_reservation(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_data_export_reservation(item: &DataExportReservation) -> f64 {
    redaction_redaction_group::global_redaction_group_weight(8, true)
}
