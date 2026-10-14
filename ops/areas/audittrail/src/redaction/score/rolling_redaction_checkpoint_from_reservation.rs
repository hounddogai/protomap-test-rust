use super::*;

use std::collections::BTreeMap;

use crate::data_export::reservation as data_export_reservation;

pub fn summarize_primary_data_export_reservations_for_checkpoint(
    items: &[data_export_reservation::PrimaryDataExportReservation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_rolling_redaction_checkpoints_from_reservation(
    items: &[data_export_reservation::PrimaryDataExportReservation],
) -> Vec<RollingRedactionCheckpoint> {
    let mut out: Vec<RollingRedactionCheckpoint> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_redaction_checkpoint(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_rolling_redaction_checkpoint(item: &RollingRedactionCheckpoint) -> f64 {
    data_export_reservation::fallback_data_export_schedule_weight(7, true)
}
