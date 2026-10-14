use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal token target.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTokenTargetView {
    pub code: String,
    pub title: String,
    pub port: String,
    pub scope: String,
    pub country: String,
    pub owner_id: String,
    pub active: String,
    pub quantity_text: String,
    pub starts_at: String,
    pub status_label: String,
}

pub fn to_seasonal_token_target_view(item: &SeasonalTokenTarget) -> SeasonalTokenTargetView {
    SeasonalTokenTargetView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 64),
        port: item.port.to_string(),
        scope: item.scope.clone(),
        country: item.country.clone(),
        owner_id: text::truncate(&item.owner_id, 120),
        active: format!("{}", item.active),
        quantity_text: item.quantity.to_string(),
        starts_at: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_token_target_views(items: &[SeasonalTokenTarget]) -> Vec<SeasonalTokenTargetView> {
    let mut views: Vec<SeasonalTokenTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_token_target_view(item));
    }
    views
}

pub fn sum_seasonal_token_target_view(view: &SeasonalTokenTargetView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_port as f64 + (parsed_quantity as f64) + (parsed_starts_at as f64))
}
