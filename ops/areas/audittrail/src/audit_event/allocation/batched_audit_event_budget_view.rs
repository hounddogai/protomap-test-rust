use super::*;

/// How screens and exports show a batched audit event budget.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAuditEventBudgetView {
    pub code: String,
    pub label_text: String,
    pub margin_text: String,
    pub threshold: String,
    pub title: String,
    pub cluster: String,
    pub cost: String,
    pub status_label: String,
}

pub fn to_batched_audit_event_budget_view(item: &BatchedAuditEventBudget) -> BatchedAuditEventBudgetView {
    BatchedAuditEventBudgetView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        margin_text: format!("{:.2}", item.margin),
        threshold: format!("{:.2}", item.threshold),
        title: item.title.clone(),
        cluster: item.cluster.clone(),
        cost: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_audit_event_budget_views(items: &[BatchedAuditEventBudget]) -> Vec<BatchedAuditEventBudgetView> {
    let mut views: Vec<BatchedAuditEventBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_audit_event_budget_view(item));
    }
    views
}

pub fn sum_batched_audit_event_budget_view(view: &BatchedAuditEventBudgetView) -> Result<f64, String> {
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_margin + parsed_threshold + parsed_cost)
}
