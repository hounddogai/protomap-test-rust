use super::*;

use acme_kit::text;

/// How screens and exports show a audit event preference.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventPreferenceView {
    pub code: String,
    pub name: String,
    pub cluster_text: String,
    pub rate_text: String,
    pub owner_id_text: String,
    pub archived_text: String,
    pub duration_minutes: String,
    pub quantity: String,
    pub unit: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_audit_event_preference_view(item: &AuditEventPreference) -> AuditEventPreferenceView {
    AuditEventPreferenceView {
        code: text::truncate(&item.code, 120),
        name: text::truncate(&item.name, 40),
        cluster_text: text::truncate(&item.cluster, 64),
        rate_text: format!("{:.2}", item.rate),
        owner_id_text: item.owner_id.clone(),
        archived_text: format!("{}", item.archived),
        duration_minutes: item.duration_minutes.to_string(),
        quantity: item.quantity.to_string(),
        unit: item.unit.clone(),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_audit_event_preference_views(items: &[AuditEventPreference]) -> Vec<AuditEventPreferenceView> {
    let mut views: Vec<AuditEventPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_audit_event_preference_view(item));
    }
    views
}

pub fn sum_audit_event_preference_view(view: &AuditEventPreferenceView) -> Result<f64, String> {
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_rate + (parsed_duration_minutes as f64) + (parsed_quantity as f64))
}
