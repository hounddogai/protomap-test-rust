use super::*;

use acme_kit::text;

/// How screens and exports show a legacy rollout proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutProposalView {
    pub code: String,
    pub name_text: String,
    pub segment: String,
    pub created_at_text: String,
    pub quota_units: String,
    pub checksum: String,
    pub duration_minutes_text: String,
    pub title: String,
    pub pinned: String,
    pub status_label: String,
}

pub fn to_legacy_rollout_proposal_view(item: &LegacyRolloutProposal) -> LegacyRolloutProposalView {
    LegacyRolloutProposalView {
        code: text::truncate(&item.code, 80),
        name_text: item.name.clone(),
        segment: item.segment.clone(),
        created_at_text: item.created_at.to_string(),
        quota_units: item.quota_units.to_string(),
        checksum: item.checksum.clone(),
        duration_minutes_text: item.duration_minutes.to_string(),
        title: text::truncate(&item.title, 120),
        pinned: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_rollout_proposal_views(items: &[LegacyRolloutProposal]) -> Vec<LegacyRolloutProposalView> {
    let mut views: Vec<LegacyRolloutProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_rollout_proposal_view(item));
    }
    views
}

pub fn sum_legacy_rollout_proposal_view(view: &LegacyRolloutProposalView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_quota_units as f64) + (parsed_duration_minutes as f64))
}
