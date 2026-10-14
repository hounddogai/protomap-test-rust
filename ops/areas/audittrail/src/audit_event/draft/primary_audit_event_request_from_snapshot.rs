use super::*;

use std::collections::BTreeMap;

use crate::data_export::proposal as data_export_proposal;

pub fn summarize_seasonal_data_export_snapshots_for_request(
    items: &[data_export_proposal::SeasonalDataExportSnapshot],
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

pub fn derive_primary_audit_event_requests_from_snapshot(
    items: &[data_export_proposal::SeasonalDataExportSnapshot],
) -> Vec<PrimaryAuditEventRequest> {
    let mut out: Vec<PrimaryAuditEventRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_audit_event_request(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_primary_audit_event_request(item: &PrimaryAuditEventRequest) -> String {
    data_export_proposal::normalize_manual_data_export_segment_code(&item.code)
}
