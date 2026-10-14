use super::*;

use std::collections::BTreeMap;

use crate::audit_event::event_request as audit_event_event_request;

pub fn summarize_capped_audit_event_requests_for_review(
    items: &[audit_event_event_request::CappedAuditEventRequest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_scheduled_data_export_reviews_from_request(
    items: &[audit_event_event_request::CappedAuditEventRequest],
) -> Vec<ScheduledDataExportReview> {
    let mut out: Vec<ScheduledDataExportReview> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_data_export_review(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_scheduled_data_export_review(item: &ScheduledDataExportReview) -> String {
    audit_event_event_request::normalize_seasonal_audit_event_notice_code(&item.code)
}
