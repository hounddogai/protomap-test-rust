use super::*;

use std::collections::BTreeMap;

use crate::data_export::targets as data_export_targets;

pub fn summarize_manual_data_export_groups_for_schedule(
    items: &[data_export_targets::ManualDataExportGroup],
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

pub fn derive_data_export_schedules_from_group(
    items: &[data_export_targets::ManualDataExportGroup],
) -> Vec<DataExportSchedule> {
    let mut out: Vec<DataExportSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_data_export_schedule(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_data_export_schedule(item: &DataExportSchedule) -> String {
    data_export_targets::default_data_export_mapping_bucket(item.net_exposure())
}
