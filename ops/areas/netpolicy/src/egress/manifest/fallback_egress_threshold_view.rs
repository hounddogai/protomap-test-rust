use super::*;

use acme_kit::text;

/// How screens and exports show a fallback egress threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEgressThresholdView {
    pub code: String,
    pub name_text: String,
    pub margin: String,
    pub label: String,
    pub archived: String,
    pub ends_at: String,
    pub status_label: String,
}

pub fn to_fallback_egress_threshold_view(item: &FallbackEgressThreshold) -> FallbackEgressThresholdView {
    FallbackEgressThresholdView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 80),
        margin: format!("{:.2}", item.margin),
        label: text::truncate(&item.label, 40),
        archived: format!("{}", item.archived),
        ends_at: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_egress_threshold_views(items: &[FallbackEgressThreshold]) -> Vec<FallbackEgressThresholdView> {
    let mut views: Vec<FallbackEgressThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_egress_threshold_view(item));
    }
    views
}

pub fn sum_fallback_egress_threshold_view(view: &FallbackEgressThresholdView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_margin + (parsed_ends_at as f64))
}
