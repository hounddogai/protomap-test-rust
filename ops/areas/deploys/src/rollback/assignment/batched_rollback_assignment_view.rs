use super::*;

use acme_kit::text;

/// How screens and exports show a batched rollback assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRollbackAssignmentView {
    pub code: String,
    pub name: String,
    pub cluster: String,
    pub quota_units: String,
    pub quantity_text: String,
    pub amount: String,
    pub notes: String,
    pub unit: String,
    pub priority: String,
    pub attempts_text: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_batched_rollback_assignment_view(item: &BatchedRollbackAssignment) -> BatchedRollbackAssignmentView {
    BatchedRollbackAssignmentView {
        code: text::truncate(&item.code, 120),
        name: text::truncate(&item.name, 120),
        cluster: item.cluster.clone(),
        quota_units: item.quota_units.to_string(),
        quantity_text: item.quantity.to_string(),
        amount: format!("{:.2}", item.amount),
        notes: item.notes.clone(),
        unit: item.unit.clone(),
        priority: item.priority.to_string(),
        attempts_text: item.attempts.to_string(),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_rollback_assignment_views(items: &[BatchedRollbackAssignment]) -> Vec<BatchedRollbackAssignmentView> {
    let mut views: Vec<BatchedRollbackAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_rollback_assignment_view(item));
    }
    views
}

pub fn sum_batched_rollback_assignment_view(view: &BatchedRollbackAssignmentView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_quantity as f64) + parsed_amount)
}
