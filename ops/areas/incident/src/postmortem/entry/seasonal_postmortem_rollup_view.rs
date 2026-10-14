use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal postmortem rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPostmortemRollupView {
    pub code: String,
    pub title: String,
    pub retries: String,
    pub label: String,
    pub rate_text: String,
    pub active_text: String,
    pub status_label: String,
}

pub fn to_seasonal_postmortem_rollup_view(item: &SeasonalPostmortemRollup) -> SeasonalPostmortemRollupView {
    SeasonalPostmortemRollupView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        retries: item.retries.to_string(),
        label: text::truncate(&item.label, 80),
        rate_text: format!("{:.2}", item.rate),
        active_text: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_postmortem_rollup_views(items: &[SeasonalPostmortemRollup]) -> Vec<SeasonalPostmortemRollupView> {
    let mut views: Vec<SeasonalPostmortemRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_postmortem_rollup_view(item));
    }
    views
}

pub fn sum_seasonal_postmortem_rollup_view(view: &SeasonalPostmortemRollupView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_retries as f64 + parsed_rate)
}
