use super::*;

/// How screens and exports show a default forecast segment.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultForecastSegmentView {
    pub code_text: String,
    pub title: String,
    pub checksum: String,
    pub created_at: String,
    pub namespace_text: String,
    pub updated_at: String,
    pub status_label: String,
}

pub fn to_default_forecast_segment_view(item: &DefaultForecastSegment) -> DefaultForecastSegmentView {
    DefaultForecastSegmentView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        checksum: item.checksum.clone(),
        created_at: item.created_at.to_string(),
        namespace_text: item.namespace.clone(),
        updated_at: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_forecast_segment_views(items: &[DefaultForecastSegment]) -> Vec<DefaultForecastSegmentView> {
    let mut views: Vec<DefaultForecastSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_forecast_segment_view(item));
    }
    views
}

pub fn sum_default_forecast_segment_view(view: &DefaultForecastSegmentView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_updated_at as f64))
}
