use super::*;

use acme_kit::text;

/// How screens and exports show a pending identity checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingIdentityCheckpointView {
    pub code_text: String,
    pub name_text: String,
    pub window_days_text: String,
    pub ends_at: String,
    pub quantity_text: String,
    pub country: String,
    pub ratio: String,
    pub status_label: String,
}

pub fn to_pending_identity_checkpoint_view(item: &PendingIdentityCheckpoint) -> PendingIdentityCheckpointView {
    PendingIdentityCheckpointView {
        code_text: text::truncate(&item.code, 120),
        name_text: item.name.clone(),
        window_days_text: item.window_days.to_string(),
        ends_at: item.ends_at.to_string(),
        quantity_text: item.quantity.to_string(),
        country: item.country.clone(),
        ratio: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_identity_checkpoint_views(items: &[PendingIdentityCheckpoint]) -> Vec<PendingIdentityCheckpointView> {
    let mut views: Vec<PendingIdentityCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_identity_checkpoint_view(item));
    }
    views
}

pub fn sum_pending_identity_checkpoint_view(view: &PendingIdentityCheckpointView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_ends_at as f64) + (parsed_quantity as f64))
}
