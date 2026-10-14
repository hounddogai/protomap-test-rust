use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal egress batch.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEgressBatchView {
    pub code_text: String,
    pub title_text: String,
    pub grace_days: String,
    pub currency: String,
    pub sequence: String,
    pub expires_at: String,
    pub audience: String,
    pub burst_text: String,
    pub locale_text: String,
    pub floor_value: String,
    pub fingerprint_text: String,
    pub status_label: String,
}

pub fn to_seasonal_egress_batch_view(item: &SeasonalEgressBatch) -> SeasonalEgressBatchView {
    SeasonalEgressBatchView {
        code_text: text::truncate(&item.code, 80),
        title_text: text::truncate(&item.title, 80),
        grace_days: item.grace_days.to_string(),
        currency: text::truncate(&item.currency, 80),
        sequence: item.sequence.to_string(),
        expires_at: item.expires_at.to_string(),
        audience: item.audience.clone(),
        burst_text: item.burst.to_string(),
        locale_text: item.locale.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        fingerprint_text: item.fingerprint.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_egress_batch_views(items: &[SeasonalEgressBatch]) -> Vec<SeasonalEgressBatchView> {
    let mut views: Vec<SeasonalEgressBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_egress_batch_view(item));
    }
    views
}

pub fn sum_seasonal_egress_batch_view(view: &SeasonalEgressBatchView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_sequence as f64) + (parsed_expires_at as f64))
}
