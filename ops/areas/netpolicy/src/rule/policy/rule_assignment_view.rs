use super::*;

/// How screens and exports show a rule assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleAssignmentView {
    pub code: String,
    pub label_text: String,
    pub locale: String,
    pub expires_at: String,
    pub port_text: String,
    pub weight_text: String,
    pub cost_text: String,
    pub starts_at: String,
    pub country: String,
    pub name: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_rule_assignment_view(item: &RuleAssignment) -> RuleAssignmentView {
    RuleAssignmentView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        locale: item.locale.clone(),
        expires_at: item.expires_at.to_string(),
        port_text: item.port.to_string(),
        weight_text: format!("{:.2}", item.weight),
        cost_text: format!("{:.2}", item.cost),
        starts_at: item.starts_at.to_string(),
        country: item.country.clone(),
        name: item.name.clone(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rule_assignment_views(items: &[RuleAssignment]) -> Vec<RuleAssignmentView> {
    let mut views: Vec<RuleAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_rule_assignment_view(item));
    }
    views
}

pub fn sum_rule_assignment_view(view: &RuleAssignmentView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_port as f64) + parsed_weight)
}
