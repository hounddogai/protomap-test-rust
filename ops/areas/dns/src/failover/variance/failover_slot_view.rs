use super::*;

use acme_kit::text;

/// How screens and exports show a failover slot.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverSlotView {
    pub code: String,
    pub label: String,
    pub cipher_text: String,
    pub source: String,
    pub priority: String,
    pub issuer: String,
    pub external_ref: String,
    pub margin: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_failover_slot_view(item: &FailoverSlot) -> FailoverSlotView {
    FailoverSlotView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 64),
        cipher_text: text::truncate(&item.cipher, 64),
        source: text::truncate(&item.source, 80),
        priority: item.priority.to_string(),
        issuer: item.issuer.clone(),
        external_ref: item.external_ref.clone(),
        margin: format!("{:.2}", item.margin),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_failover_slot_views(items: &[FailoverSlot]) -> Vec<FailoverSlotView> {
    let mut views: Vec<FailoverSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_failover_slot_view(item));
    }
    views
}

pub fn sum_failover_slot_view(view: &FailoverSlotView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_priority as f64 + parsed_margin + parsed_amount)
}
