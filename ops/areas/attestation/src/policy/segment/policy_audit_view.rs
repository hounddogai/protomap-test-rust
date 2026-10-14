use super::*;

/// How screens and exports show a policy audit.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyAuditView {
    pub code_text: String,
    pub name_text: String,
    pub cost: String,
    pub active: String,
    pub attempts_text: String,
    pub batch_size: String,
    pub maximum: String,
    pub minimum: String,
    pub threshold: String,
    pub status_label: String,
}

pub fn to_policy_audit_view(item: &PolicyAudit) -> PolicyAuditView {
    PolicyAuditView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        cost: format!("{:.2}", item.cost),
        active: format!("{}", item.active),
        attempts_text: item.attempts.to_string(),
        batch_size: item.batch_size.to_string(),
        maximum: format!("{:.2}", item.maximum),
        minimum: format!("{:.2}", item.minimum),
        threshold: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_policy_audit_views(items: &[PolicyAudit]) -> Vec<PolicyAuditView> {
    let mut views: Vec<PolicyAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_policy_audit_view(item));
    }
    views
}

pub fn sum_policy_audit_view(view: &PolicyAuditView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_cost + (parsed_attempts as f64) + (parsed_batch_size as f64))
}
