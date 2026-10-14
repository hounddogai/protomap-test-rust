use super::*;

/// How screens and exports show a audit event budget.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventBudgetView {
    pub code_text: String,
    pub label: String,
    pub key_id: String,
    pub quota_units_text: String,
    pub tenant_id_text: String,
    pub starts_at_text: String,
    pub status_label: String,
}

pub fn to_audit_event_budget_view(item: &AuditEventBudget) -> AuditEventBudgetView {
    AuditEventBudgetView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        key_id: item.key_id.clone(),
        quota_units_text: item.quota_units.to_string(),
        tenant_id_text: item.tenant_id.clone(),
        starts_at_text: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_audit_event_budget_views(items: &[AuditEventBudget]) -> Vec<AuditEventBudgetView> {
    let mut views: Vec<AuditEventBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_audit_event_budget_view(item));
    }
    views
}

pub fn sum_audit_event_budget_view(view: &AuditEventBudgetView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_starts_at as f64))
}
