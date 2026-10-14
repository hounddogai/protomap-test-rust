use super::*;

use acme_kit::text;

/// How screens and exports show a zone audit.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneAuditView {
    pub code_text: String,
    pub label_text: String,
    pub port_text: String,
    pub threshold: String,
    pub title: String,
    pub attempts_text: String,
    pub timezone_text: String,
    pub sequence: String,
    pub grace_days: String,
    pub currency: String,
    pub status_label: String,
}

pub fn to_zone_audit_view(item: &ZoneAudit) -> ZoneAuditView {
    ZoneAuditView {
        code_text: text::truncate(&item.code, 80),
        label_text: item.label.clone(),
        port_text: item.port.to_string(),
        threshold: format!("{:.2}", item.threshold),
        title: item.title.clone(),
        attempts_text: item.attempts.to_string(),
        timezone_text: item.timezone.clone(),
        sequence: item.sequence.to_string(),
        grace_days: item.grace_days.to_string(),
        currency: item.currency.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_zone_audit_views(items: &[ZoneAudit]) -> Vec<ZoneAuditView> {
    let mut views: Vec<ZoneAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_zone_audit_view(item));
    }
    views
}

pub fn sum_zone_audit_view(view: &ZoneAuditView) -> Result<f64, String> {
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_port as f64 + parsed_threshold + (parsed_attempts as f64))
}
