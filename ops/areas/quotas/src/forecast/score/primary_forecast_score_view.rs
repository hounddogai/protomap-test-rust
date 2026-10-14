use super::*;

use acme_kit::text;

/// How screens and exports show a primary forecast score.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryForecastScoreView {
    pub code: String,
    pub label: String,
    pub version_text: String,
    pub cluster_text: String,
    pub score_text: String,
    pub description_text: String,
    pub starts_at_text: String,
    pub issuer: String,
    pub title_text: String,
    pub attempts_text: String,
    pub status_label: String,
}

pub fn to_primary_forecast_score_view(item: &PrimaryForecastScore) -> PrimaryForecastScoreView {
    PrimaryForecastScoreView {
        code: text::truncate(&item.code, 80),
        label: text::truncate(&item.label, 120),
        version_text: item.version.to_string(),
        cluster_text: item.cluster.clone(),
        score_text: format!("{:.2}", item.score),
        description_text: text::truncate(&item.description, 120),
        starts_at_text: item.starts_at.to_string(),
        issuer: item.issuer.clone(),
        title_text: item.title.clone(),
        attempts_text: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_forecast_score_views(items: &[PrimaryForecastScore]) -> Vec<PrimaryForecastScoreView> {
    let mut views: Vec<PrimaryForecastScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_forecast_score_view(item));
    }
    views
}

pub fn sum_primary_forecast_score_view(view: &PrimaryForecastScoreView) -> Result<f64, String> {
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_version as f64 + parsed_score + (parsed_starts_at as f64))
}
