use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal health check segment.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalHealthCheckSegmentView {
    pub code: String,
    pub label: String,
    pub burst_text: String,
    pub cluster: String,
    pub locale_text: String,
    pub minimum: String,
    pub limit: String,
    pub timezone: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_seasonal_health_check_segment_view(item: &SeasonalHealthCheckSegment) -> SeasonalHealthCheckSegmentView {
    SeasonalHealthCheckSegmentView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 120),
        burst_text: item.burst.to_string(),
        cluster: item.cluster.clone(),
        locale_text: text::truncate(&item.locale, 120),
        minimum: format!("{:.2}", item.minimum),
        limit: item.limit.to_string(),
        timezone: item.timezone.clone(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_health_check_segment_views(
    items: &[SeasonalHealthCheckSegment],
) -> Vec<SeasonalHealthCheckSegmentView> {
    let mut views: Vec<SeasonalHealthCheckSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_health_check_segment_view(item));
    }
    views
}

pub fn sum_seasonal_health_check_segment_view(view: &SeasonalHealthCheckSegmentView) -> Result<f64, String> {
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_burst as f64 + parsed_minimum + (parsed_limit as f64))
}
