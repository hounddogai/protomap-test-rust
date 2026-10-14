use super::*;

use std::collections::BTreeMap;

use crate::data_export::draft as data_export_draft;

pub fn summarize_seasonal_data_export_tiers_for_segment(
    items: &[data_export_draft::SeasonalDataExportTier],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_rolling_audit_event_segments_from_tier(
    items: &[data_export_draft::SeasonalDataExportTier],
) -> Vec<RollingAuditEventSegment> {
    let mut out: Vec<RollingAuditEventSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_audit_event_segment(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_rolling_audit_event_segment(item: &RollingAuditEventSegment) -> String {
    data_export_draft::seasonal_data_export_tier_bucket(item.projected_load())
}
