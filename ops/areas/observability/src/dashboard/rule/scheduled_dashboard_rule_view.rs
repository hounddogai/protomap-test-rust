use super::*;

/// How screens and exports show a scheduled dashboard rule.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDashboardRuleView {
    pub code: String,
    pub label: String,
    pub contact_email: String,
    pub capacity: String,
    pub grace_days: String,
    pub batch_size: String,
    pub minimum: String,
    pub principal: String,
    pub category: String,
    pub port: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_scheduled_dashboard_rule_view(item: &ScheduledDashboardRule) -> ScheduledDashboardRuleView {
    ScheduledDashboardRuleView {
        code: item.code.clone(),
        label: item.label.clone(),
        contact_email: item.contact_email.clone(),
        capacity: item.capacity.to_string(),
        grace_days: item.grace_days.to_string(),
        batch_size: item.batch_size.to_string(),
        minimum: format!("{:.2}", item.minimum),
        principal: item.principal.clone(),
        category: item.category.clone(),
        port: item.port.to_string(),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_dashboard_rule_views(items: &[ScheduledDashboardRule]) -> Vec<ScheduledDashboardRuleView> {
    let mut views: Vec<ScheduledDashboardRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_dashboard_rule_view(item));
    }
    views
}

pub fn sum_scheduled_dashboard_rule_view(view: &ScheduledDashboardRuleView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_grace_days as f64) + (parsed_batch_size as f64))
}
