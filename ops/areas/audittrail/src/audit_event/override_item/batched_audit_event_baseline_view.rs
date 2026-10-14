use super::*;

use acme_kit::text;

/// How screens and exports show a batched audit event baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAuditEventBaselineView {
    pub code: String,
    pub title: String,
    pub amount: String,
    pub principal: String,
    pub owner_id_text: String,
    pub minimum_text: String,
    pub updated_at_text: String,
    pub external_ref: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_batched_audit_event_baseline_view(item: &BatchedAuditEventBaseline) -> BatchedAuditEventBaselineView {
    BatchedAuditEventBaselineView {
        code: item.code.clone(),
        title: item.title.clone(),
        amount: format!("{:.2}", item.amount),
        principal: text::truncate(&item.principal, 64),
        owner_id_text: text::truncate(&item.owner_id, 64),
        minimum_text: format!("{:.2}", item.minimum),
        updated_at_text: item.updated_at.to_string(),
        external_ref: item.external_ref.clone(),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_audit_event_baseline_views(
    items: &[BatchedAuditEventBaseline],
) -> Vec<BatchedAuditEventBaselineView> {
    let mut views: Vec<BatchedAuditEventBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_audit_event_baseline_view(item));
    }
    views
}

pub fn sum_batched_audit_event_baseline_view(view: &BatchedAuditEventBaselineView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_amount + parsed_minimum + (parsed_updated_at as f64))
}
