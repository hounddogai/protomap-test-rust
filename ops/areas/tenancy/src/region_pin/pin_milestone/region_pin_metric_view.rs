use super::*;

use acme_kit::text;

/// How screens and exports show a region pin metric.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinMetricView {
    pub code: String,
    pub label: String,
    pub port: String,
    pub fingerprint_text: String,
    pub score: String,
    pub pinned: String,
    pub unit_text: String,
    pub batch_size_text: String,
    pub reviewer_text: String,
    pub status_label: String,
}

pub fn to_region_pin_metric_view(item: &RegionPinMetric) -> RegionPinMetricView {
    RegionPinMetricView {
        code: item.code.clone(),
        label: item.label.clone(),
        port: item.port.to_string(),
        fingerprint_text: text::truncate(&item.fingerprint, 40),
        score: format!("{:.2}", item.score),
        pinned: format!("{}", item.pinned),
        unit_text: text::truncate(&item.unit, 80),
        batch_size_text: item.batch_size.to_string(),
        reviewer_text: text::truncate(&item.reviewer, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_region_pin_metric_views(items: &[RegionPinMetric]) -> Vec<RegionPinMetricView> {
    let mut views: Vec<RegionPinMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_region_pin_metric_view(item));
    }
    views
}

pub fn sum_region_pin_metric_view(view: &RegionPinMetricView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_port as f64 + parsed_score + (parsed_batch_size as f64))
}
