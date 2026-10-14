use super::*;

use acme_kit::text;

/// How screens and exports show a weighted budget burn forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBudgetBurnForecastView {
    pub code_text: String,
    pub name_text: String,
    pub retries: String,
    pub ttl_seconds_text: String,
    pub namespace: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_weighted_budget_burn_forecast_view(item: &WeightedBudgetBurnForecast) -> WeightedBudgetBurnForecastView {
    WeightedBudgetBurnForecastView {
        code_text: text::truncate(&item.code, 120),
        name_text: text::truncate(&item.name, 120),
        retries: item.retries.to_string(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        namespace: text::truncate(&item.namespace, 80),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_budget_burn_forecast_views(
    items: &[WeightedBudgetBurnForecast],
) -> Vec<WeightedBudgetBurnForecastView> {
    let mut views: Vec<WeightedBudgetBurnForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_budget_burn_forecast_view(item));
    }
    views
}

pub fn sum_weighted_budget_burn_forecast_view(view: &WeightedBudgetBurnForecastView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_ttl_seconds as f64) + (parsed_port as f64))
}
