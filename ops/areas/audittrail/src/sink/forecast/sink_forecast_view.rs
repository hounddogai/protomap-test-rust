use super::*;

use acme_kit::text;

/// How screens and exports show a sink forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkForecastView {
    pub code_text: String,
    pub label: String,
    pub starts_at: String,
    pub channel: String,
    pub fingerprint: String,
    pub status_label: String,
}

pub fn to_sink_forecast_view(item: &SinkForecast) -> SinkForecastView {
    SinkForecastView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 64),
        starts_at: item.starts_at.to_string(),
        channel: item.channel.clone(),
        fingerprint: text::truncate(&item.fingerprint, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_sink_forecast_views(items: &[SinkForecast]) -> Vec<SinkForecastView> {
    let mut views: Vec<SinkForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_sink_forecast_view(item));
    }
    views
}

pub fn sum_sink_forecast_view(view: &SinkForecastView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_starts_at as f64)
}
