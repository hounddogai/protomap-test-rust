use super::*;

use acme_kit::text;

/// How screens and exports show a segment score.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentScoreView {
    pub code: String,
    pub label_text: String,
    pub window_days: String,
    pub cost: String,
    pub maximum: String,
    pub unit: String,
    pub version: String,
    pub status_label: String,
}

pub fn to_segment_score_view(item: &SegmentScore) -> SegmentScoreView {
    SegmentScoreView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 80),
        window_days: item.window_days.to_string(),
        cost: format!("{:.2}", item.cost),
        maximum: format!("{:.2}", item.maximum),
        unit: item.unit.clone(),
        version: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_segment_score_views(items: &[SegmentScore]) -> Vec<SegmentScoreView> {
    let mut views: Vec<SegmentScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_segment_score_view(item));
    }
    views
}

pub fn sum_segment_score_view(view: &SegmentScoreView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_window_days as f64 + parsed_cost + parsed_maximum)
}
