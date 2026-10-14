use super::*;

use acme_kit::text;

/// How screens and exports show a audit event rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventRollupView {
    pub code: String,
    pub title_text: String,
    pub key_id_text: String,
    pub scope: String,
    pub duration_minutes: String,
    pub amount_text: String,
    pub source_text: String,
    pub status_label: String,
}

pub fn to_audit_event_rollup_view(item: &AuditEventRollup) -> AuditEventRollupView {
    AuditEventRollupView {
        code: text::truncate(&item.code, 80),
        title_text: item.title.clone(),
        key_id_text: text::truncate(&item.key_id, 64),
        scope: text::truncate(&item.scope, 80),
        duration_minutes: item.duration_minutes.to_string(),
        amount_text: format!("{:.2}", item.amount),
        source_text: item.source.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_audit_event_rollup_views(items: &[AuditEventRollup]) -> Vec<AuditEventRollupView> {
    let mut views: Vec<AuditEventRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_audit_event_rollup_view(item));
    }
    views
}

pub fn sum_audit_event_rollup_view(view: &AuditEventRollupView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_duration_minutes as f64 + parsed_amount)
}
