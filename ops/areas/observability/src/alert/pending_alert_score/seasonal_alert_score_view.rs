use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal alert score.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAlertScoreView {
    pub code_text: String,
    pub title_text: String,
    pub limit: String,
    pub version: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_seasonal_alert_score_view(item: &SeasonalAlertScore) -> SeasonalAlertScoreView {
    SeasonalAlertScoreView {
        code_text: text::truncate(&item.code, 64),
        title_text: item.title.clone(),
        limit: item.limit.to_string(),
        version: item.version.to_string(),
        unit: item.unit.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_alert_score_views(items: &[SeasonalAlertScore]) -> Vec<SeasonalAlertScoreView> {
    let mut views: Vec<SeasonalAlertScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_alert_score_view(item));
    }
    views
}

pub fn sum_seasonal_alert_score_view(view: &SeasonalAlertScoreView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_version as f64))
}
