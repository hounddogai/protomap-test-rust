use super::*;

use acme_kit::text;

/// How screens and exports show a escalation allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationAllocationView {
    pub code: String,
    pub name: String,
    pub expires_at: String,
    pub maximum_text: String,
    pub owner_id_text: String,
    pub status_label: String,
}

pub fn to_escalation_allocation_view(item: &EscalationAllocation) -> EscalationAllocationView {
    EscalationAllocationView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 40),
        expires_at: item.expires_at.to_string(),
        maximum_text: format!("{:.2}", item.maximum),
        owner_id_text: text::truncate(&item.owner_id, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_escalation_allocation_views(items: &[EscalationAllocation]) -> Vec<EscalationAllocationView> {
    let mut views: Vec<EscalationAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_escalation_allocation_view(item));
    }
    views
}

pub fn sum_escalation_allocation_view(view: &EscalationAllocationView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_expires_at as f64 + parsed_maximum)
}
