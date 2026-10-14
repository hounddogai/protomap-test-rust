use super::*;

use acme_kit::text;

/// How screens and exports show a batched approval checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedApprovalCheckpointView {
    pub code: String,
    pub name: String,
    pub country_text: String,
    pub capacity: String,
    pub checksum_text: String,
    pub visible: String,
    pub label: String,
    pub title: String,
    pub status_label: String,
}

pub fn to_batched_approval_checkpoint_view(item: &BatchedApprovalCheckpoint) -> BatchedApprovalCheckpointView {
    BatchedApprovalCheckpointView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        country_text: item.country.clone(),
        capacity: item.capacity.to_string(),
        checksum_text: text::truncate(&item.checksum, 64),
        visible: format!("{}", item.visible),
        label: text::truncate(&item.label, 40),
        title: item.title.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_approval_checkpoint_views(items: &[BatchedApprovalCheckpoint]) -> Vec<BatchedApprovalCheckpointView> {
    let mut views: Vec<BatchedApprovalCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_approval_checkpoint_view(item));
    }
    views
}

pub fn sum_batched_approval_checkpoint_view(view: &BatchedApprovalCheckpointView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_capacity as f64)
}
