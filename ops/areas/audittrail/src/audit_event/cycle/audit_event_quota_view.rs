use super::*;

use acme_kit::text;

/// How screens and exports show a audit event quota.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventQuotaView {
    pub code: String,
    pub label: String,
    pub grace_days: String,
    pub approved_text: String,
    pub segment: String,
    pub name: String,
    pub starts_at_text: String,
    pub retries: String,
    pub contact_email: String,
    pub status_label: String,
}

pub fn to_audit_event_quota_view(item: &AuditEventQuota) -> AuditEventQuotaView {
    AuditEventQuotaView {
        code: item.code.clone(),
        label: item.label.clone(),
        grace_days: item.grace_days.to_string(),
        approved_text: format!("{}", item.approved),
        segment: text::truncate(&item.segment, 40),
        name: text::truncate(&item.name, 80),
        starts_at_text: item.starts_at.to_string(),
        retries: item.retries.to_string(),
        contact_email: item.contact_email.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_audit_event_quota_views(items: &[AuditEventQuota]) -> Vec<AuditEventQuotaView> {
    let mut views: Vec<AuditEventQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_audit_event_quota_view(item));
    }
    views
}

pub fn sum_audit_event_quota_view(view: &AuditEventQuotaView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_starts_at as f64) + (parsed_retries as f64))
}
