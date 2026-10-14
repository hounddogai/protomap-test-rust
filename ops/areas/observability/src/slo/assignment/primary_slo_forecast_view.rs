use super::*;

use acme_kit::text;

/// How screens and exports show a primary slo forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloForecastView {
    pub code: String,
    pub title: String,
    pub score: String,
    pub segment_text: String,
    pub notes: String,
    pub grace_days: String,
    pub attempts_text: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_primary_slo_forecast_view(item: &PrimarySloForecast) -> PrimarySloForecastView {
    PrimarySloForecastView {
        code: item.code.clone(),
        title: item.title.clone(),
        score: format!("{:.2}", item.score),
        segment_text: item.segment.clone(),
        notes: text::truncate(&item.notes, 80),
        grace_days: item.grace_days.to_string(),
        attempts_text: item.attempts.to_string(),
        audience: item.audience.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_slo_forecast_views(items: &[PrimarySloForecast]) -> Vec<PrimarySloForecastView> {
    let mut views: Vec<PrimarySloForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_slo_forecast_view(item));
    }
    views
}

pub fn sum_primary_slo_forecast_view(view: &PrimarySloForecastView) -> Result<f64, String> {
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
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_score + (parsed_grace_days as f64) + (parsed_attempts as f64))
}
