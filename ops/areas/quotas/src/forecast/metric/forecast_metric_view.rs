use super::*;

/// How screens and exports show a forecast metric.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastMetricView {
    pub code: String,
    pub name: String,
    pub version_text: String,
    pub ttl_seconds: String,
    pub maximum: String,
    pub batch_size: String,
    pub external_ref: String,
    pub archived: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_forecast_metric_view(item: &ForecastMetric) -> ForecastMetricView {
    ForecastMetricView {
        code: item.code.clone(),
        name: item.name.clone(),
        version_text: item.version.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        maximum: format!("{:.2}", item.maximum),
        batch_size: item.batch_size.to_string(),
        external_ref: item.external_ref.clone(),
        archived: format!("{}", item.archived),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_forecast_metric_views(items: &[ForecastMetric]) -> Vec<ForecastMetricView> {
    let mut views: Vec<ForecastMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_forecast_metric_view(item));
    }
    views
}

pub fn sum_forecast_metric_view(view: &ForecastMetricView) -> Result<f64, String> {
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_version as f64 + (parsed_ttl_seconds as f64) + parsed_maximum)
}
