use super::*;

use acme_kit::text;

/// How screens and exports show a batched policy forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyForecastView {
    pub code: String,
    pub title_text: String,
    pub ceiling_text: String,
    pub reviewer_text: String,
    pub ttl_seconds: String,
    pub status_label: String,
}

pub fn to_batched_policy_forecast_view(item: &BatchedPolicyForecast) -> BatchedPolicyForecastView {
    BatchedPolicyForecastView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 120),
        ceiling_text: format!("{:.2}", item.ceiling),
        reviewer_text: item.reviewer.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_policy_forecast_views(items: &[BatchedPolicyForecast]) -> Vec<BatchedPolicyForecastView> {
    let mut views: Vec<BatchedPolicyForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_policy_forecast_view(item));
    }
    views
}

pub fn sum_batched_policy_forecast_view(view: &BatchedPolicyForecastView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_ceiling + (parsed_ttl_seconds as f64))
}
