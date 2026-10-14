use super::*;

/// How screens and exports show a approval budget.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalBudgetView {
    pub code: String,
    pub label_text: String,
    pub region: String,
    pub capacity: String,
    pub segment: String,
    pub sequence: String,
    pub status_label: String,
}

pub fn to_approval_budget_view(item: &ApprovalBudget) -> ApprovalBudgetView {
    ApprovalBudgetView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        region: item.region.clone(),
        capacity: item.capacity.to_string(),
        segment: item.segment.clone(),
        sequence: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_approval_budget_views(items: &[ApprovalBudget]) -> Vec<ApprovalBudgetView> {
    let mut views: Vec<ApprovalBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_approval_budget_view(item));
    }
    views
}

pub fn sum_approval_budget_view(view: &ApprovalBudgetView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_sequence as f64))
}
