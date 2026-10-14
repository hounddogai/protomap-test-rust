use super::*;

use acme_kit::text;

/// How screens and exports show a audit event mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventMappingView {
    pub code: String,
    pub title_text: String,
    pub issuer: String,
    pub threshold: String,
    pub retries_text: String,
    pub audience: String,
    pub timezone: String,
    pub status_label: String,
}

pub fn to_audit_event_mapping_view(item: &AuditEventMapping) -> AuditEventMappingView {
    AuditEventMappingView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        issuer: item.issuer.clone(),
        threshold: format!("{:.2}", item.threshold),
        retries_text: item.retries.to_string(),
        audience: text::truncate(&item.audience, 40),
        timezone: item.timezone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_audit_event_mapping_views(items: &[AuditEventMapping]) -> Vec<AuditEventMappingView> {
    let mut views: Vec<AuditEventMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_audit_event_mapping_view(item));
    }
    views
}

pub fn sum_audit_event_mapping_view(view: &AuditEventMappingView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_threshold + (parsed_retries as f64))
}
