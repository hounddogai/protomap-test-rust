use super::*;

use acme_kit::text;

/// How screens and exports show a forecast item.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastItemView {
    pub code: String,
    pub label: String,
    pub rate: String,
    pub host: String,
    pub audience: String,
    pub duration_minutes: String,
    pub limit: String,
    pub archived: String,
    pub status_label: String,
}

pub fn to_forecast_item_view(item: &ForecastItem) -> ForecastItemView {
    ForecastItemView {
        code: item.code.clone(),
        label: item.label.clone(),
        rate: format!("{:.2}", item.rate),
        host: item.host.clone(),
        audience: text::truncate(&item.audience, 64),
        duration_minutes: item.duration_minutes.to_string(),
        limit: item.limit.to_string(),
        archived: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_forecast_item_views(items: &[ForecastItem]) -> Vec<ForecastItemView> {
    let mut views: Vec<ForecastItemView> = Vec::new();
    for item in items.iter() {
        views.push(to_forecast_item_view(item));
    }
    views
}

pub fn sum_forecast_item_view(view: &ForecastItemView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_rate + (parsed_duration_minutes as f64) + (parsed_limit as f64))
}
