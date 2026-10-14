use super::*;

use acme_kit::text;

/// How screens and exports show a capped segment estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedSegmentEstimateView {
    pub code_text: String,
    pub label: String,
    pub ends_at: String,
    pub retries: String,
    pub region: String,
    pub margin: String,
    pub scope: String,
    pub unit_text: String,
    pub phone: String,
    pub country: String,
    pub status_label: String,
}

pub fn to_capped_segment_estimate_view(item: &CappedSegmentEstimate) -> CappedSegmentEstimateView {
    CappedSegmentEstimateView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        ends_at: item.ends_at.to_string(),
        retries: item.retries.to_string(),
        region: item.region.clone(),
        margin: format!("{:.2}", item.margin),
        scope: item.scope.clone(),
        unit_text: text::truncate(&item.unit, 40),
        phone: item.phone.clone(),
        country: item.country.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_segment_estimate_views(items: &[CappedSegmentEstimate]) -> Vec<CappedSegmentEstimateView> {
    let mut views: Vec<CappedSegmentEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_segment_estimate_view(item));
    }
    views
}

pub fn sum_capped_segment_estimate_view(view: &CappedSegmentEstimateView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_retries as f64) + parsed_margin)
}
