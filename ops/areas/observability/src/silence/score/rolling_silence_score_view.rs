use super::*;

/// How screens and exports show a rolling silence score.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingSilenceScoreView {
    pub code_text: String,
    pub name_text: String,
    pub window_days: String,
    pub quota_units: String,
    pub ends_at_text: String,
    pub currency: String,
    pub status_label: String,
}

pub fn to_rolling_silence_score_view(item: &RollingSilenceScore) -> RollingSilenceScoreView {
    RollingSilenceScoreView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        window_days: item.window_days.to_string(),
        quota_units: item.quota_units.to_string(),
        ends_at_text: item.ends_at.to_string(),
        currency: item.currency.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_silence_score_views(items: &[RollingSilenceScore]) -> Vec<RollingSilenceScoreView> {
    let mut views: Vec<RollingSilenceScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_silence_score_view(item));
    }
    views
}

pub fn sum_rolling_silence_score_view(view: &RollingSilenceScoreView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_quota_units as f64) + (parsed_ends_at as f64))
}
