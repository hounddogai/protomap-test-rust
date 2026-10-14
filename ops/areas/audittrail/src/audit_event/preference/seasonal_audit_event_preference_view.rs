use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal audit event preference.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAuditEventPreferenceView {
    pub code: String,
    pub name_text: String,
    pub unit: String,
    pub locale: String,
    pub minimum: String,
    pub ttl_seconds_text: String,
    pub principal_text: String,
    pub rotation_days: String,
    pub category_text: String,
    pub amount: String,
    pub priority_text: String,
    pub status_label: String,
}

pub fn to_seasonal_audit_event_preference_view(
    item: &SeasonalAuditEventPreference,
) -> SeasonalAuditEventPreferenceView {
    SeasonalAuditEventPreferenceView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        unit: text::truncate(&item.unit, 64),
        locale: item.locale.clone(),
        minimum: format!("{:.2}", item.minimum),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        principal_text: item.principal.clone(),
        rotation_days: item.rotation_days.to_string(),
        category_text: item.category.clone(),
        amount: format!("{:.2}", item.amount),
        priority_text: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_audit_event_preference_views(
    items: &[SeasonalAuditEventPreference],
) -> Vec<SeasonalAuditEventPreferenceView> {
    let mut views: Vec<SeasonalAuditEventPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_audit_event_preference_view(item));
    }
    views
}

pub fn sum_seasonal_audit_event_preference_view(view: &SeasonalAuditEventPreferenceView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_minimum + (parsed_ttl_seconds as f64) + (parsed_rotation_days as f64))
}
