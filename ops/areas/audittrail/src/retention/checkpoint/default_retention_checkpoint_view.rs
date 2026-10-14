use super::*;

use acme_kit::text;

/// How screens and exports show a default retention checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRetentionCheckpointView {
    pub code: String,
    pub title: String,
    pub ratio: String,
    pub category_text: String,
    pub quantity: String,
    pub fingerprint: String,
    pub revision: String,
    pub limit: String,
    pub budget_text: String,
    pub status_label: String,
}

pub fn to_default_retention_checkpoint_view(item: &DefaultRetentionCheckpoint) -> DefaultRetentionCheckpointView {
    DefaultRetentionCheckpointView {
        code: item.code.clone(),
        title: item.title.clone(),
        ratio: format!("{:.2}", item.ratio),
        category_text: item.category.clone(),
        quantity: item.quantity.to_string(),
        fingerprint: text::truncate(&item.fingerprint, 120),
        revision: item.revision.to_string(),
        limit: item.limit.to_string(),
        budget_text: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_retention_checkpoint_views(
    items: &[DefaultRetentionCheckpoint],
) -> Vec<DefaultRetentionCheckpointView> {
    let mut views: Vec<DefaultRetentionCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_retention_checkpoint_view(item));
    }
    views
}

pub fn sum_default_retention_checkpoint_view(view: &DefaultRetentionCheckpointView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_ratio + (parsed_quantity as f64) + (parsed_revision as f64))
}
