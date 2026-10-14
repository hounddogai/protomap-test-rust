use super::*;

use acme_kit::text;

/// How screens and exports show a fallback zone segment.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackZoneSegmentView {
    pub code: String,
    pub title: String,
    pub amount: String,
    pub window_days: String,
    pub visible_text: String,
    pub status_label: String,
}

pub fn to_fallback_zone_segment_view(item: &FallbackZoneSegment) -> FallbackZoneSegmentView {
    FallbackZoneSegmentView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 80),
        amount: format!("{:.2}", item.amount),
        window_days: item.window_days.to_string(),
        visible_text: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_zone_segment_views(items: &[FallbackZoneSegment]) -> Vec<FallbackZoneSegmentView> {
    let mut views: Vec<FallbackZoneSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_zone_segment_view(item));
    }
    views
}

pub fn sum_fallback_zone_segment_view(view: &FallbackZoneSegmentView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_amount + (parsed_window_days as f64))
}
