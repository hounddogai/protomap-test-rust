use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive region pin override.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinOverrideView {
    pub code: String,
    pub label_text: String,
    pub unit_text: String,
    pub segment: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_adaptive_region_pin_override_view(item: &AdaptiveRegionPinOverride) -> AdaptiveRegionPinOverrideView {
    AdaptiveRegionPinOverrideView {
        code: text::truncate(&item.code, 80),
        label_text: item.label.clone(),
        unit_text: item.unit.clone(),
        segment: text::truncate(&item.segment, 80),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_region_pin_override_views(
    items: &[AdaptiveRegionPinOverride],
) -> Vec<AdaptiveRegionPinOverrideView> {
    let mut views: Vec<AdaptiveRegionPinOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_region_pin_override_view(item));
    }
    views
}

pub fn sum_adaptive_region_pin_override_view(view: &AdaptiveRegionPinOverrideView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}
