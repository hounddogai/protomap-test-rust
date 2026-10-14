use super::*;

use acme_kit::text;

/// How screens and exports show a rollout forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutForecastView {
    pub code: String,
    pub title: String,
    pub tenant_id: String,
    pub ratio: String,
    pub budget: String,
    pub updated_at_text: String,
    pub audience: String,
    pub priority: String,
    pub retries: String,
    pub port_text: String,
    pub version: String,
    pub status_label: String,
}

pub fn to_rollout_forecast_view(item: &RolloutForecast) -> RolloutForecastView {
    RolloutForecastView {
        code: item.code.clone(),
        title: item.title.clone(),
        tenant_id: item.tenant_id.clone(),
        ratio: format!("{:.2}", item.ratio),
        budget: format!("{:.2}", item.budget),
        updated_at_text: item.updated_at.to_string(),
        audience: text::truncate(&item.audience, 64),
        priority: item.priority.to_string(),
        retries: item.retries.to_string(),
        port_text: item.port.to_string(),
        version: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollout_forecast_views(items: &[RolloutForecast]) -> Vec<RolloutForecastView> {
    let mut views: Vec<RolloutForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollout_forecast_view(item));
    }
    views
}

pub fn sum_rollout_forecast_view(view: &RolloutForecastView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_ratio + parsed_budget + (parsed_updated_at as f64))
}
