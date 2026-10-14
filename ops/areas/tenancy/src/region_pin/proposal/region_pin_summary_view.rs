use super::*;

use acme_kit::text;

/// How screens and exports show a region pin summary.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinSummaryView {
    pub code: String,
    pub label: String,
    pub channel_text: String,
    pub description: String,
    pub floor_value_text: String,
    pub threshold: String,
    pub score_text: String,
    pub status_label: String,
}

pub fn to_region_pin_summary_view(item: &RegionPinSummary) -> RegionPinSummaryView {
    RegionPinSummaryView {
        code: item.code.clone(),
        label: item.label.clone(),
        channel_text: item.channel.clone(),
        description: text::truncate(&item.description, 80),
        floor_value_text: format!("{:.2}", item.floor_value),
        threshold: format!("{:.2}", item.threshold),
        score_text: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_region_pin_summary_views(items: &[RegionPinSummary]) -> Vec<RegionPinSummaryView> {
    let mut views: Vec<RegionPinSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_region_pin_summary_view(item));
    }
    views
}

pub fn sum_region_pin_summary_view(view: &RegionPinSummaryView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_floor_value + parsed_threshold + parsed_score)
}
