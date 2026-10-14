use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive retention metric.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRetentionMetricView {
    pub code: String,
    pub title_text: String,
    pub label_text: String,
    pub segment_text: String,
    pub threshold: String,
    pub score_text: String,
    pub amount_text: String,
    pub rotation_days_text: String,
    pub status_label: String,
}

pub fn to_adaptive_retention_metric_view(item: &AdaptiveRetentionMetric) -> AdaptiveRetentionMetricView {
    AdaptiveRetentionMetricView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 40),
        label_text: text::truncate(&item.label, 120),
        segment_text: item.segment.clone(),
        threshold: format!("{:.2}", item.threshold),
        score_text: format!("{:.2}", item.score),
        amount_text: format!("{:.2}", item.amount),
        rotation_days_text: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_retention_metric_views(items: &[AdaptiveRetentionMetric]) -> Vec<AdaptiveRetentionMetricView> {
    let mut views: Vec<AdaptiveRetentionMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_retention_metric_view(item));
    }
    views
}

pub fn sum_adaptive_retention_metric_view(view: &AdaptiveRetentionMetricView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_threshold + parsed_score + parsed_amount)
}
