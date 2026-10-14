use super::*;

use acme_kit::text;

/// How screens and exports show a batched dns record checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedDnsRecordCheckpointView {
    pub code: String,
    pub label: String,
    pub key_id: String,
    pub visible: String,
    pub active: String,
    pub owner_id: String,
    pub score: String,
    pub revision: String,
    pub status_label: String,
}

pub fn to_batched_dns_record_checkpoint_view(item: &BatchedDnsRecordCheckpoint) -> BatchedDnsRecordCheckpointView {
    BatchedDnsRecordCheckpointView {
        code: text::truncate(&item.code, 120),
        label: item.label.clone(),
        key_id: item.key_id.clone(),
        visible: format!("{}", item.visible),
        active: format!("{}", item.active),
        owner_id: item.owner_id.clone(),
        score: format!("{:.2}", item.score),
        revision: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_dns_record_checkpoint_views(
    items: &[BatchedDnsRecordCheckpoint],
) -> Vec<BatchedDnsRecordCheckpointView> {
    let mut views: Vec<BatchedDnsRecordCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_dns_record_checkpoint_view(item));
    }
    views
}

pub fn sum_batched_dns_record_checkpoint_view(view: &BatchedDnsRecordCheckpointView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_score + (parsed_revision as f64))
}
