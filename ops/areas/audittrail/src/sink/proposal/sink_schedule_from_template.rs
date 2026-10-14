use super::*;

use std::collections::BTreeMap;

use crate::data_export::policy as data_export_policy;

pub fn summarize_scheduled_data_export_templates_for_schedule(
    items: &[data_export_policy::ScheduledDataExportTemplate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_sink_schedules_from_template(
    items: &[data_export_policy::ScheduledDataExportTemplate],
) -> Vec<SinkSchedule> {
    let mut out: Vec<SinkSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_sink_schedule(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_sink_schedule(item: &SinkSchedule) -> String {
    data_export_policy::normalize_data_export_policy_code(&item.code)
}
