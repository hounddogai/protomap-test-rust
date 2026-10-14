use super::*;

use acme_kit::text;

/// How screens and exports show a capped flow log agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedFlowLogAgreementView {
    pub code_text: String,
    pub label: String,
    pub quantity_text: String,
    pub active: String,
    pub capacity: String,
    pub grace_days: String,
    pub external_ref: String,
    pub scope: String,
    pub reviewer: String,
    pub contact_email: String,
    pub rate_text: String,
    pub status_label: String,
}

pub fn to_capped_flow_log_agreement_view(item: &CappedFlowLogAgreement) -> CappedFlowLogAgreementView {
    CappedFlowLogAgreementView {
        code_text: text::truncate(&item.code, 40),
        label: text::truncate(&item.label, 120),
        quantity_text: item.quantity.to_string(),
        active: format!("{}", item.active),
        capacity: item.capacity.to_string(),
        grace_days: item.grace_days.to_string(),
        external_ref: item.external_ref.clone(),
        scope: text::truncate(&item.scope, 40),
        reviewer: text::truncate(&item.reviewer, 64),
        contact_email: text::truncate(&item.contact_email, 64),
        rate_text: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_flow_log_agreement_views(items: &[CappedFlowLogAgreement]) -> Vec<CappedFlowLogAgreementView> {
    let mut views: Vec<CappedFlowLogAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_flow_log_agreement_view(item));
    }
    views
}

pub fn sum_capped_flow_log_agreement_view(view: &CappedFlowLogAgreementView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
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
    Ok(parsed_quantity as f64 + (parsed_capacity as f64) + (parsed_grace_days as f64))
}
