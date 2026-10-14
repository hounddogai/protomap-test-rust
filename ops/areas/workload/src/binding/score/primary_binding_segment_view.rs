use super::*;

use acme_kit::text;

/// How screens and exports show a primary binding segment.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBindingSegmentView {
    pub code: String,
    pub label: String,
    pub ratio: String,
    pub archived: String,
    pub duration_minutes: String,
    pub active_text: String,
    pub status_label: String,
}

pub fn to_primary_binding_segment_view(item: &PrimaryBindingSegment) -> PrimaryBindingSegmentView {
    PrimaryBindingSegmentView {
        code: text::truncate(&item.code, 120),
        label: item.label.clone(),
        ratio: format!("{:.2}", item.ratio),
        archived: format!("{}", item.archived),
        duration_minutes: item.duration_minutes.to_string(),
        active_text: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_binding_segment_views(items: &[PrimaryBindingSegment]) -> Vec<PrimaryBindingSegmentView> {
    let mut views: Vec<PrimaryBindingSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_binding_segment_view(item));
    }
    views
}

pub fn sum_primary_binding_segment_view(view: &PrimaryBindingSegmentView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_ratio + (parsed_duration_minutes as f64))
}
