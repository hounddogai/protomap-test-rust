use super::*;

/// How screens and exports show a tiered alert audit.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertAuditView {
    pub code_text: String,
    pub name_text: String,
    pub starts_at: String,
    pub created_at: String,
    pub capacity_text: String,
    pub scope_text: String,
    pub timezone: String,
    pub quota_units: String,
    pub source: String,
    pub status_label: String,
}

pub fn to_tiered_alert_audit_view(item: &TieredAlertAudit) -> TieredAlertAuditView {
    TieredAlertAuditView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        starts_at: item.starts_at.to_string(),
        created_at: item.created_at.to_string(),
        capacity_text: item.capacity.to_string(),
        scope_text: item.scope.clone(),
        timezone: item.timezone.clone(),
        quota_units: item.quota_units.to_string(),
        source: item.source.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_alert_audit_views(items: &[TieredAlertAudit]) -> Vec<TieredAlertAuditView> {
    let mut views: Vec<TieredAlertAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_alert_audit_view(item));
    }
    views
}

pub fn sum_tiered_alert_audit_view(view: &TieredAlertAuditView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_starts_at as f64 + (parsed_created_at as f64) + (parsed_capacity as f64))
}
