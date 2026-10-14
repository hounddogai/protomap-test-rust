use super::*;

use acme_kit::text;

/// How screens and exports show a lenient timeline forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientTimelineForecastView {
    pub code_text: String,
    pub title_text: String,
    pub duration_minutes: String,
    pub visible: String,
    pub window_days: String,
    pub unit_text: String,
    pub locale: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_lenient_timeline_forecast_view(item: &LenientTimelineForecast) -> LenientTimelineForecastView {
    LenientTimelineForecastView {
        code_text: item.code.clone(),
        title_text: text::truncate(&item.title, 80),
        duration_minutes: item.duration_minutes.to_string(),
        visible: format!("{}", item.visible),
        window_days: item.window_days.to_string(),
        unit_text: item.unit.clone(),
        locale: text::truncate(&item.locale, 120),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_timeline_forecast_views(items: &[LenientTimelineForecast]) -> Vec<LenientTimelineForecastView> {
    let mut views: Vec<LenientTimelineForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_timeline_forecast_view(item));
    }
    views
}

pub fn sum_lenient_timeline_forecast_view(view: &LenientTimelineForecastView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_window_days as f64) + (parsed_port as f64))
}
