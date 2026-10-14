use super::*;

use acme_kit::text;

/// How screens and exports show a tiered rollout proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutProposalView {
    pub code: String,
    pub label_text: String,
    pub weight: String,
    pub cipher: String,
    pub starts_at: String,
    pub phone: String,
    pub key_id: String,
    pub reviewer_text: String,
    pub status_label: String,
}

pub fn to_tiered_rollout_proposal_view(item: &TieredRolloutProposal) -> TieredRolloutProposalView {
    TieredRolloutProposalView {
        code: text::truncate(&item.code, 64),
        label_text: text::truncate(&item.label, 40),
        weight: format!("{:.2}", item.weight),
        cipher: text::truncate(&item.cipher, 80),
        starts_at: item.starts_at.to_string(),
        phone: text::truncate(&item.phone, 64),
        key_id: item.key_id.clone(),
        reviewer_text: item.reviewer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_rollout_proposal_views(items: &[TieredRolloutProposal]) -> Vec<TieredRolloutProposalView> {
    let mut views: Vec<TieredRolloutProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_rollout_proposal_view(item));
    }
    views
}

pub fn sum_tiered_rollout_proposal_view(view: &TieredRolloutProposalView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_weight + (parsed_starts_at as f64))
}
