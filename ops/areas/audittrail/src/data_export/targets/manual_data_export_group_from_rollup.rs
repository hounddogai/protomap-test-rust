use super::*;

use std::collections::BTreeMap;

use crate::audit_event::legacy_event_rollup as audit_event_legacy_event_rollup;

pub fn summarize_global_audit_event_rollups_for_group(
    items: &[audit_event_legacy_event_rollup::GlobalAuditEventRollup],
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

pub fn derive_manual_data_export_groups_from_rollup(
    items: &[audit_event_legacy_event_rollup::GlobalAuditEventRollup],
) -> Vec<ManualDataExportGroup> {
    let mut out: Vec<ManualDataExportGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_data_export_group(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn derive_manual_data_export_group(item: &ManualDataExportGroup) -> String {
    audit_event_legacy_event_rollup::classify_audit_event_summary(item.adjusted_volume())
}
