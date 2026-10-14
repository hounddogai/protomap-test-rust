use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal overage estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalOverageEstimateView {
    pub code: String,
    pub title: String,
    pub rate: String,
    pub name_text: String,
    pub scope: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_seasonal_overage_estimate_view(item: &SeasonalOverageEstimate) -> SeasonalOverageEstimateView {
    SeasonalOverageEstimateView {
        code: text::truncate(&item.code, 80),
        title: text::truncate(&item.title, 80),
        rate: format!("{:.2}", item.rate),
        name_text: text::truncate(&item.name, 120),
        scope: item.scope.clone(),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_overage_estimate_views(items: &[SeasonalOverageEstimate]) -> Vec<SeasonalOverageEstimateView> {
    let mut views: Vec<SeasonalOverageEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_overage_estimate_view(item));
    }
    views
}

pub fn sum_seasonal_overage_estimate_view(view: &SeasonalOverageEstimateView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_rate + (parsed_window_days as f64))
}
