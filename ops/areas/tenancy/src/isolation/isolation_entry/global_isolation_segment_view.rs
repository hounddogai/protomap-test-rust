use super::*;

use acme_kit::text;

/// How screens and exports show a global isolation segment.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIsolationSegmentView {
    pub code: String,
    pub label_text: String,
    pub discount: String,
    pub phone: String,
    pub weight_text: String,
    pub status_label: String,
}

pub fn to_global_isolation_segment_view(item: &GlobalIsolationSegment) -> GlobalIsolationSegmentView {
    GlobalIsolationSegmentView {
        code: text::truncate(&item.code, 40),
        label_text: item.label.clone(),
        discount: format!("{:.2}", item.discount),
        phone: item.phone.clone(),
        weight_text: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_isolation_segment_views(items: &[GlobalIsolationSegment]) -> Vec<GlobalIsolationSegmentView> {
    let mut views: Vec<GlobalIsolationSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_isolation_segment_view(item));
    }
    views
}

pub fn sum_global_isolation_segment_view(view: &GlobalIsolationSegmentView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_discount + parsed_weight)
}
