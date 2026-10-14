use super::*;

use acme_kit::text;

/// How screens and exports show a peering record.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringRecordView {
    pub code: String,
    pub label: String,
    pub principal_text: String,
    pub retries: String,
    pub fingerprint: String,
    pub category_text: String,
    pub attempts: String,
    pub quantity: String,
    pub ratio: String,
    pub status_label: String,
}

pub fn to_peering_record_view(item: &PeeringRecord) -> PeeringRecordView {
    PeeringRecordView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 64),
        principal_text: item.principal.clone(),
        retries: item.retries.to_string(),
        fingerprint: item.fingerprint.clone(),
        category_text: item.category.clone(),
        attempts: item.attempts.to_string(),
        quantity: item.quantity.to_string(),
        ratio: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_peering_record_views(items: &[PeeringRecord]) -> Vec<PeeringRecordView> {
    let mut views: Vec<PeeringRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_peering_record_view(item));
    }
    views
}

pub fn sum_peering_record_view(view: &PeeringRecordView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_attempts as f64) + (parsed_quantity as f64))
}
