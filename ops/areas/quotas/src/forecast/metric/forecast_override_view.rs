use super::*;

use acme_kit::text;

/// How screens and exports show a forecast override.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastOverrideView {
    pub code: String,
    pub title_text: String,
    pub active: String,
    pub rate: String,
    pub updated_at: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_forecast_override_view(item: &ForecastOverride) -> ForecastOverrideView {
    ForecastOverrideView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 120),
        active: format!("{}", item.active),
        rate: format!("{:.2}", item.rate),
        updated_at: item.updated_at.to_string(),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_forecast_override_views(items: &[ForecastOverride]) -> Vec<ForecastOverrideView> {
    let mut views: Vec<ForecastOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_forecast_override_view(item));
    }
    views
}

pub fn sum_forecast_override_view(view: &ForecastOverrideView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_rate + (parsed_updated_at as f64) + (parsed_window_days as f64))
}
