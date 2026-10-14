use super::*;

use acme_kit::text;

/// How screens and exports show a manual allowlist forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualAllowlistForecastView {
    pub code_text: String,
    pub title_text: String,
    pub namespace_text: String,
    pub score: String,
    pub unit: String,
    pub checksum: String,
    pub host_text: String,
    pub reviewer: String,
    pub grace_days: String,
    pub limit_text: String,
    pub status_label: String,
}

pub fn to_manual_allowlist_forecast_view(item: &ManualAllowlistForecast) -> ManualAllowlistForecastView {
    ManualAllowlistForecastView {
        code_text: text::truncate(&item.code, 40),
        title_text: item.title.clone(),
        namespace_text: item.namespace.clone(),
        score: format!("{:.2}", item.score),
        unit: item.unit.clone(),
        checksum: item.checksum.clone(),
        host_text: item.host.clone(),
        reviewer: item.reviewer.clone(),
        grace_days: item.grace_days.to_string(),
        limit_text: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_allowlist_forecast_views(items: &[ManualAllowlistForecast]) -> Vec<ManualAllowlistForecastView> {
    let mut views: Vec<ManualAllowlistForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_allowlist_forecast_view(item));
    }
    views
}

pub fn sum_manual_allowlist_forecast_view(view: &ManualAllowlistForecastView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_score + (parsed_grace_days as f64) + (parsed_limit as f64))
}
