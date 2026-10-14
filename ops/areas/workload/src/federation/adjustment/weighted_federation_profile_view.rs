use super::*;

use acme_kit::text;

/// How screens and exports show a weighted federation profile.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFederationProfileView {
    pub code_text: String,
    pub name: String,
    pub batch_size: String,
    pub floor_value: String,
    pub quantity: String,
    pub notes: String,
    pub maximum: String,
    pub pinned: String,
    pub threshold_text: String,
    pub status_label: String,
}

pub fn to_weighted_federation_profile_view(item: &WeightedFederationProfile) -> WeightedFederationProfileView {
    WeightedFederationProfileView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        batch_size: item.batch_size.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        quantity: item.quantity.to_string(),
        notes: text::truncate(&item.notes, 120),
        maximum: format!("{:.2}", item.maximum),
        pinned: format!("{}", item.pinned),
        threshold_text: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_federation_profile_views(items: &[WeightedFederationProfile]) -> Vec<WeightedFederationProfileView> {
    let mut views: Vec<WeightedFederationProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_federation_profile_view(item));
    }
    views
}

pub fn sum_weighted_federation_profile_view(view: &WeightedFederationProfileView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_batch_size as f64 + parsed_floor_value + (parsed_quantity as f64))
}
