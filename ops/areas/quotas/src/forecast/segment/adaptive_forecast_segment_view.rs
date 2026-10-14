use super::*;

/// How screens and exports show a adaptive forecast segment.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveForecastSegmentView {
    pub code_text: String,
    pub title_text: String,
    pub namespace_text: String,
    pub host: String,
    pub window_days: String,
    pub active: String,
    pub archived_text: String,
    pub status_label: String,
}

pub fn to_adaptive_forecast_segment_view(item: &AdaptiveForecastSegment) -> AdaptiveForecastSegmentView {
    AdaptiveForecastSegmentView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        namespace_text: item.namespace.clone(),
        host: item.host.clone(),
        window_days: item.window_days.to_string(),
        active: format!("{}", item.active),
        archived_text: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_forecast_segment_views(items: &[AdaptiveForecastSegment]) -> Vec<AdaptiveForecastSegmentView> {
    let mut views: Vec<AdaptiveForecastSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_forecast_segment_view(item));
    }
    views
}

pub fn sum_adaptive_forecast_segment_view(view: &AdaptiveForecastSegmentView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_window_days as f64)
}
