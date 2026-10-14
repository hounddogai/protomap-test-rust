use super::*;

use acme_kit::text;

/// How screens and exports show a tiered peering override.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringOverrideView {
    pub code_text: String,
    pub label: String,
    pub source: String,
    pub scope_text: String,
    pub capacity: String,
    pub category_text: String,
    pub discount: String,
    pub duration_minutes: String,
    pub description: String,
    pub status_label: String,
}

pub fn to_tiered_peering_override_view(item: &TieredPeeringOverride) -> TieredPeeringOverrideView {
    TieredPeeringOverrideView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        source: item.source.clone(),
        scope_text: item.scope.clone(),
        capacity: item.capacity.to_string(),
        category_text: item.category.clone(),
        discount: format!("{:.2}", item.discount),
        duration_minutes: item.duration_minutes.to_string(),
        description: text::truncate(&item.description, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_peering_override_views(items: &[TieredPeeringOverride]) -> Vec<TieredPeeringOverrideView> {
    let mut views: Vec<TieredPeeringOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_peering_override_view(item));
    }
    views
}

pub fn sum_tiered_peering_override_view(view: &TieredPeeringOverrideView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_capacity as f64 + parsed_discount + (parsed_duration_minutes as f64))
}
