use super::*;

use std::collections::BTreeMap;

use crate::data_export::quota as data_export_quota;

pub fn summarize_data_export_estimates_for_assignment(
    items: &[data_export_quota::DataExportEstimate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_audit_event_assignments_from_estimate(
    items: &[data_export_quota::DataExportEstimate],
) -> Vec<AuditEventAssignment> {
    let mut out: Vec<AuditEventAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_audit_event_assignment(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_audit_event_assignment(item: &AuditEventAssignment) -> String {
    data_export_quota::normalize_seasonal_data_export_mapping_code(&item.code)
}
