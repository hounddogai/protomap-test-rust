use super::*;

use acme_kit::text;

/// How screens and exports show a manual sink request.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualSinkRequestView {
    pub code: String,
    pub title: String,
    pub description: String,
    pub floor_value: String,
    pub segment: String,
    pub status_label: String,
}

pub fn to_manual_sink_request_view(item: &ManualSinkRequest) -> ManualSinkRequestView {
    ManualSinkRequestView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        description: item.description.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        segment: item.segment.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_sink_request_views(items: &[ManualSinkRequest]) -> Vec<ManualSinkRequestView> {
    let mut views: Vec<ManualSinkRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_sink_request_view(item));
    }
    views
}

pub fn sum_manual_sink_request_view(view: &ManualSinkRequestView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_floor_value)
}
