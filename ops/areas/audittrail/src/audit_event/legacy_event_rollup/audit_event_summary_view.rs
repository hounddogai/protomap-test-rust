use super::*;

use acme_kit::text;

/// How screens and exports show a audit event summary.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventSummaryView {
    pub code: String,
    pub name: String,
    pub priority_text: String,
    pub burst: String,
    pub starts_at: String,
    pub rate: String,
    pub discount: String,
    pub retries: String,
    pub phone: String,
    pub key_id: String,
    pub status_label: String,
}

pub fn to_audit_event_summary_view(item: &AuditEventSummary) -> AuditEventSummaryView {
    AuditEventSummaryView {
        code: item.code.clone(),
        name: item.name.clone(),
        priority_text: item.priority.to_string(),
        burst: item.burst.to_string(),
        starts_at: item.starts_at.to_string(),
        rate: format!("{:.2}", item.rate),
        discount: format!("{:.2}", item.discount),
        retries: item.retries.to_string(),
        phone: text::truncate(&item.phone, 80),
        key_id: item.key_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_audit_event_summary_views(items: &[AuditEventSummary]) -> Vec<AuditEventSummaryView> {
    let mut views: Vec<AuditEventSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_audit_event_summary_view(item));
    }
    views
}

pub fn sum_audit_event_summary_view(view: &AuditEventSummaryView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_burst as f64) + (parsed_starts_at as f64))
}
