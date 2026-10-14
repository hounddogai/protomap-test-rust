use super::*;

use acme_kit::text;

/// How screens and exports show a fallback reservation forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReservationForecastView {
    pub code_text: String,
    pub title: String,
    pub unit: String,
    pub cipher: String,
    pub created_at: String,
    pub capacity: String,
    pub audience_text: String,
    pub window_days_text: String,
    pub cost_text: String,
    pub status_label: String,
}

pub fn to_fallback_reservation_forecast_view(item: &FallbackReservationForecast) -> FallbackReservationForecastView {
    FallbackReservationForecastView {
        code_text: text::truncate(&item.code, 64),
        title: text::truncate(&item.title, 64),
        unit: text::truncate(&item.unit, 40),
        cipher: item.cipher.clone(),
        created_at: item.created_at.to_string(),
        capacity: item.capacity.to_string(),
        audience_text: text::truncate(&item.audience, 40),
        window_days_text: item.window_days.to_string(),
        cost_text: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_reservation_forecast_views(
    items: &[FallbackReservationForecast],
) -> Vec<FallbackReservationForecastView> {
    let mut views: Vec<FallbackReservationForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_reservation_forecast_view(item));
    }
    views
}

pub fn sum_fallback_reservation_forecast_view(view: &FallbackReservationForecastView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_capacity as f64) + (parsed_window_days as f64))
}
