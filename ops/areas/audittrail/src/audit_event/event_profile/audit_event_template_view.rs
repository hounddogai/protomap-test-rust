use super::*;

use acme_kit::text;

/// How screens and exports show a audit event template.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventTemplateView {
    pub code_text: String,
    pub name_text: String,
    pub active_text: String,
    pub host: String,
    pub issuer: String,
    pub notes: String,
    pub revision_text: String,
    pub status_label: String,
}

pub fn to_audit_event_template_view(item: &AuditEventTemplate) -> AuditEventTemplateView {
    AuditEventTemplateView {
        code_text: item.code.clone(),
        name_text: text::truncate(&item.name, 40),
        active_text: format!("{}", item.active),
        host: item.host.clone(),
        issuer: text::truncate(&item.issuer, 40),
        notes: item.notes.clone(),
        revision_text: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_audit_event_template_views(items: &[AuditEventTemplate]) -> Vec<AuditEventTemplateView> {
    let mut views: Vec<AuditEventTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_audit_event_template_view(item));
    }
    views
}

pub fn sum_audit_event_template_view(view: &AuditEventTemplateView) -> Result<f64, String> {
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_revision as f64)
}
