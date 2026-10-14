use super::*;

use acme_kit::text;

/// How screens and exports show a audit event profile.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventProfileView {
    pub code_text: String,
    pub title: String,
    pub priority_text: String,
    pub ttl_seconds: String,
    pub amount_text: String,
    pub floor_value_text: String,
    pub status_label: String,
}

pub fn to_audit_event_profile_view(item: &AuditEventProfile) -> AuditEventProfileView {
    AuditEventProfileView {
        code_text: text::truncate(&item.code, 64),
        title: text::truncate(&item.title, 40),
        priority_text: item.priority.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        amount_text: format!("{:.2}", item.amount),
        floor_value_text: format!("{:.2}", item.floor_value),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_audit_event_profile_views(items: &[AuditEventProfile]) -> Vec<AuditEventProfileView> {
    let mut views: Vec<AuditEventProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_audit_event_profile_view(item));
    }
    views
}

pub fn sum_audit_event_profile_view(view: &AuditEventProfileView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_ttl_seconds as f64) + parsed_amount)
}
