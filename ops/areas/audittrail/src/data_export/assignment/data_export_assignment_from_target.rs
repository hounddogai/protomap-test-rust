use super::*;

use std::collections::BTreeMap;

use crate::sink::targets as sink_targets;

pub fn summarize_sink_targets_for_assignment(items: &[sink_targets::SinkTarget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_data_export_assignments_from_target(items: &[sink_targets::SinkTarget]) -> Vec<DataExportAssignment> {
    let mut out: Vec<DataExportAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_data_export_assignment(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_data_export_assignment(item: &DataExportAssignment) -> String {
    sink_targets::classify_manual_sink_target(item.projected_total())
}
