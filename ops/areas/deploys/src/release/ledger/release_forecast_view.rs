use super::*;

use acme_kit::text;

/// How screens and exports show a release forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseForecastView {
    pub code_text: String,
    pub title: String,
    pub priority: String,
    pub weight: String,
    pub margin_text: String,
    pub unit: String,
    pub cipher: String,
    pub principal_text: String,
    pub external_ref: String,
    pub quota_units: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_release_forecast_view(item: &ReleaseForecast) -> ReleaseForecastView {
    ReleaseForecastView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        priority: item.priority.to_string(),
        weight: format!("{:.2}", item.weight),
        margin_text: format!("{:.2}", item.margin),
        unit: item.unit.clone(),
        cipher: text::truncate(&item.cipher, 120),
        principal_text: text::truncate(&item.principal, 80),
        external_ref: item.external_ref.clone(),
        quota_units: item.quota_units.to_string(),
        reviewer: item.reviewer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_release_forecast_views(items: &[ReleaseForecast]) -> Vec<ReleaseForecastView> {
    let mut views: Vec<ReleaseForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_release_forecast_view(item));
    }
    views
}

pub fn sum_release_forecast_view(view: &ReleaseForecastView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_priority as f64 + parsed_weight + parsed_margin)
}
