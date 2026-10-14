use super::*;

use acme_kit::text;

/// How screens and exports show a fallback policy forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPolicyForecastView {
    pub code: String,
    pub name: String,
    pub quantity: String,
    pub score: String,
    pub duration_minutes: String,
    pub region_text: String,
    pub tenant_id_text: String,
    pub status_label: String,
}

pub fn to_fallback_policy_forecast_view(item: &FallbackPolicyForecast) -> FallbackPolicyForecastView {
    FallbackPolicyForecastView {
        code: item.code.clone(),
        name: item.name.clone(),
        quantity: item.quantity.to_string(),
        score: format!("{:.2}", item.score),
        duration_minutes: item.duration_minutes.to_string(),
        region_text: text::truncate(&item.region, 64),
        tenant_id_text: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_policy_forecast_views(items: &[FallbackPolicyForecast]) -> Vec<FallbackPolicyForecastView> {
    let mut views: Vec<FallbackPolicyForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_policy_forecast_view(item));
    }
    views
}

pub fn sum_fallback_policy_forecast_view(view: &FallbackPolicyForecastView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_score + (parsed_duration_minutes as f64))
}
