use super::*;

use acme_kit::text;

/// How screens and exports show a measurement segment.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementSegmentView {
    pub code: String,
    pub title: String,
    pub budget: String,
    pub category: String,
    pub name_text: String,
    pub channel: String,
    pub expires_at: String,
    pub label: String,
    pub status_label: String,
}

pub fn to_measurement_segment_view(item: &MeasurementSegment) -> MeasurementSegmentView {
    MeasurementSegmentView {
        code: text::truncate(&item.code, 40),
        title: item.title.clone(),
        budget: format!("{:.2}", item.budget),
        category: text::truncate(&item.category, 80),
        name_text: text::truncate(&item.name, 40),
        channel: text::truncate(&item.channel, 80),
        expires_at: item.expires_at.to_string(),
        label: text::truncate(&item.label, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_measurement_segment_views(items: &[MeasurementSegment]) -> Vec<MeasurementSegmentView> {
    let mut views: Vec<MeasurementSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_measurement_segment_view(item));
    }
    views
}

pub fn sum_measurement_segment_view(view: &MeasurementSegmentView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_budget + (parsed_expires_at as f64))
}
