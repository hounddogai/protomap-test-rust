use super::*;

use acme_kit::text;

/// How screens and exports show a primary audit event request.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryAuditEventRequestView {
    pub code: String,
    pub name_text: String,
    pub window_days: String,
    pub expires_at: String,
    pub status_label: String,
}

pub fn to_primary_audit_event_request_view(item: &PrimaryAuditEventRequest) -> PrimaryAuditEventRequestView {
    PrimaryAuditEventRequestView {
        code: text::truncate(&item.code, 80),
        name_text: item.name.clone(),
        window_days: item.window_days.to_string(),
        expires_at: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_audit_event_request_views(items: &[PrimaryAuditEventRequest]) -> Vec<PrimaryAuditEventRequestView> {
    let mut views: Vec<PrimaryAuditEventRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_audit_event_request_view(item));
    }
    views
}

pub fn sum_primary_audit_event_request_view(view: &PrimaryAuditEventRequestView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_expires_at as f64))
}
