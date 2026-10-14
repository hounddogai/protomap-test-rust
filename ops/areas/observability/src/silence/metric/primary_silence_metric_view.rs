use super::*;

use acme_kit::text;

/// How screens and exports show a primary silence metric.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySilenceMetricView {
    pub code: String,
    pub label_text: String,
    pub expires_at: String,
    pub external_ref: String,
    pub channel_text: String,
    pub phone: String,
    pub locale: String,
    pub priority: String,
    pub status_label: String,
}

pub fn to_primary_silence_metric_view(item: &PrimarySilenceMetric) -> PrimarySilenceMetricView {
    PrimarySilenceMetricView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 80),
        expires_at: item.expires_at.to_string(),
        external_ref: item.external_ref.clone(),
        channel_text: item.channel.clone(),
        phone: text::truncate(&item.phone, 64),
        locale: item.locale.clone(),
        priority: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_silence_metric_views(items: &[PrimarySilenceMetric]) -> Vec<PrimarySilenceMetricView> {
    let mut views: Vec<PrimarySilenceMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_silence_metric_view(item));
    }
    views
}

pub fn sum_primary_silence_metric_view(view: &PrimarySilenceMetricView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_priority as f64))
}
