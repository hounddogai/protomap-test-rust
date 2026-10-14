use super::*;

use acme_kit::text;

/// How screens and exports show a forecast segment.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastSegmentView {
    pub code: String,
    pub title_text: String,
    pub quota_units: String,
    pub phone_text: String,
    pub budget: String,
    pub notes: String,
    pub status_label: String,
}

pub fn to_forecast_segment_view(item: &ForecastSegment) -> ForecastSegmentView {
    ForecastSegmentView {
        code: text::truncate(&item.code, 40),
        title_text: text::truncate(&item.title, 120),
        quota_units: item.quota_units.to_string(),
        phone_text: text::truncate(&item.phone, 64),
        budget: format!("{:.2}", item.budget),
        notes: text::truncate(&item.notes, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_forecast_segment_views(items: &[ForecastSegment]) -> Vec<ForecastSegmentView> {
    let mut views: Vec<ForecastSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_forecast_segment_view(item));
    }
    views
}

pub fn sum_forecast_segment_view(view: &ForecastSegmentView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_budget)
}
