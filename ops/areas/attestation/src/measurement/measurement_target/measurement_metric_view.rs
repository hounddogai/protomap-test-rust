use super::*;

use acme_kit::text;

/// How screens and exports show a measurement metric.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementMetricView {
    pub code: String,
    pub title: String,
    pub rotation_days: String,
    pub source: String,
    pub port: String,
    pub summary: String,
    pub scope: String,
    pub status_label: String,
}

pub fn to_measurement_metric_view(item: &MeasurementMetric) -> MeasurementMetricView {
    MeasurementMetricView {
        code: item.code.clone(),
        title: item.title.clone(),
        rotation_days: item.rotation_days.to_string(),
        source: item.source.clone(),
        port: item.port.to_string(),
        summary: item.summary.clone(),
        scope: text::truncate(&item.scope, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_measurement_metric_views(items: &[MeasurementMetric]) -> Vec<MeasurementMetricView> {
    let mut views: Vec<MeasurementMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_measurement_metric_view(item));
    }
    views
}

pub fn sum_measurement_metric_view(view: &MeasurementMetricView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_port as f64))
}
