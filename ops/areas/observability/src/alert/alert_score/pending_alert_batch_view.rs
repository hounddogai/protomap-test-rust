use super::*;

use acme_kit::text;

/// How screens and exports show a pending alert batch.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingAlertBatchView {
    pub code: String,
    pub title: String,
    pub retries: String,
    pub channel_text: String,
    pub sequence: String,
    pub notes: String,
    pub principal_text: String,
    pub quantity: String,
    pub country: String,
    pub threshold_text: String,
    pub status_label: String,
}

pub fn to_pending_alert_batch_view(item: &PendingAlertBatch) -> PendingAlertBatchView {
    PendingAlertBatchView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        retries: item.retries.to_string(),
        channel_text: item.channel.clone(),
        sequence: item.sequence.to_string(),
        notes: item.notes.clone(),
        principal_text: item.principal.clone(),
        quantity: item.quantity.to_string(),
        country: item.country.clone(),
        threshold_text: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_alert_batch_views(items: &[PendingAlertBatch]) -> Vec<PendingAlertBatchView> {
    let mut views: Vec<PendingAlertBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_alert_batch_view(item));
    }
    views
}

pub fn sum_pending_alert_batch_view(view: &PendingAlertBatchView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_sequence as f64) + (parsed_quantity as f64))
}
