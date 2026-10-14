use super::*;

use acme_kit::text;

/// How screens and exports show a manual action item checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualActionItemCheckpointView {
    pub code: String,
    pub title: String,
    pub pinned: String,
    pub checksum: String,
    pub ends_at_text: String,
    pub weight: String,
    pub status_label: String,
}

pub fn to_manual_action_item_checkpoint_view(item: &ManualActionItemCheckpoint) -> ManualActionItemCheckpointView {
    ManualActionItemCheckpointView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 64),
        pinned: format!("{}", item.pinned),
        checksum: text::truncate(&item.checksum, 40),
        ends_at_text: item.ends_at.to_string(),
        weight: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_action_item_checkpoint_views(
    items: &[ManualActionItemCheckpoint],
) -> Vec<ManualActionItemCheckpointView> {
    let mut views: Vec<ManualActionItemCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_action_item_checkpoint_view(item));
    }
    views
}

pub fn sum_manual_action_item_checkpoint_view(view: &ManualActionItemCheckpointView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_ends_at as f64 + parsed_weight)
}
