use super::*;

use acme_kit::text;

/// How screens and exports show a weighted segment forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentForecastView {
    pub code: String,
    pub title: String,
    pub sequence: String,
    pub timezone_text: String,
    pub maximum: String,
    pub tenant_id: String,
    pub rotation_days: String,
    pub priority: String,
    pub status_label: String,
}

pub fn to_weighted_segment_forecast_view(item: &WeightedSegmentForecast) -> WeightedSegmentForecastView {
    WeightedSegmentForecastView {
        code: item.code.clone(),
        title: item.title.clone(),
        sequence: item.sequence.to_string(),
        timezone_text: text::truncate(&item.timezone, 80),
        maximum: format!("{:.2}", item.maximum),
        tenant_id: item.tenant_id.clone(),
        rotation_days: item.rotation_days.to_string(),
        priority: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_segment_forecast_views(items: &[WeightedSegmentForecast]) -> Vec<WeightedSegmentForecastView> {
    let mut views: Vec<WeightedSegmentForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_segment_forecast_view(item));
    }
    views
}

pub fn sum_weighted_segment_forecast_view(view: &WeightedSegmentForecastView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_sequence as f64 + parsed_maximum + (parsed_rotation_days as f64))
}
