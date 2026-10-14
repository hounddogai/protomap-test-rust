use super::*;

use acme_kit::text;

/// How screens and exports show a rolling flow log forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingFlowLogForecastView {
    pub code: String,
    pub name: String,
    pub host: String,
    pub cluster: String,
    pub updated_at_text: String,
    pub principal: String,
    pub priority_text: String,
    pub ttl_seconds: String,
    pub starts_at_text: String,
    pub status_label: String,
}

pub fn to_rolling_flow_log_forecast_view(item: &RollingFlowLogForecast) -> RollingFlowLogForecastView {
    RollingFlowLogForecastView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 40),
        host: item.host.clone(),
        cluster: item.cluster.clone(),
        updated_at_text: item.updated_at.to_string(),
        principal: item.principal.clone(),
        priority_text: item.priority.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        starts_at_text: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_flow_log_forecast_views(items: &[RollingFlowLogForecast]) -> Vec<RollingFlowLogForecastView> {
    let mut views: Vec<RollingFlowLogForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_flow_log_forecast_view(item));
    }
    views
}

pub fn sum_rolling_flow_log_forecast_view(view: &RollingFlowLogForecastView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_priority as f64) + (parsed_ttl_seconds as f64))
}
