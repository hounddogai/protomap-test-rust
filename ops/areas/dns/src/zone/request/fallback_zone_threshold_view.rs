use super::*;

use acme_kit::text;

/// How screens and exports show a fallback zone threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackZoneThresholdView {
    pub code: String,
    pub name: String,
    pub segment: String,
    pub key_id_text: String,
    pub summary: String,
    pub quantity: String,
    pub sequence: String,
    pub audience: String,
    pub ends_at: String,
    pub archived: String,
    pub floor_value: String,
    pub status_label: String,
}

pub fn to_fallback_zone_threshold_view(item: &FallbackZoneThreshold) -> FallbackZoneThresholdView {
    FallbackZoneThresholdView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        segment: text::truncate(&item.segment, 120),
        key_id_text: text::truncate(&item.key_id, 40),
        summary: item.summary.clone(),
        quantity: item.quantity.to_string(),
        sequence: item.sequence.to_string(),
        audience: item.audience.clone(),
        ends_at: item.ends_at.to_string(),
        archived: format!("{}", item.archived),
        floor_value: format!("{:.2}", item.floor_value),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_zone_threshold_views(items: &[FallbackZoneThreshold]) -> Vec<FallbackZoneThresholdView> {
    let mut views: Vec<FallbackZoneThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_zone_threshold_view(item));
    }
    views
}

pub fn sum_fallback_zone_threshold_view(view: &FallbackZoneThresholdView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_sequence as f64) + (parsed_ends_at as f64))
}
