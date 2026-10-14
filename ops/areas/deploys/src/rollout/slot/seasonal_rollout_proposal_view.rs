use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal rollout proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRolloutProposalView {
    pub code_text: String,
    pub name: String,
    pub contact_email_text: String,
    pub host: String,
    pub pinned: String,
    pub ratio_text: String,
    pub scope: String,
    pub tenant_id: String,
    pub rotation_days: String,
    pub source: String,
    pub status_label: String,
}

pub fn to_seasonal_rollout_proposal_view(item: &SeasonalRolloutProposal) -> SeasonalRolloutProposalView {
    SeasonalRolloutProposalView {
        code_text: text::truncate(&item.code, 64),
        name: item.name.clone(),
        contact_email_text: item.contact_email.clone(),
        host: text::truncate(&item.host, 80),
        pinned: format!("{}", item.pinned),
        ratio_text: format!("{:.2}", item.ratio),
        scope: item.scope.clone(),
        tenant_id: item.tenant_id.clone(),
        rotation_days: item.rotation_days.to_string(),
        source: item.source.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_rollout_proposal_views(items: &[SeasonalRolloutProposal]) -> Vec<SeasonalRolloutProposalView> {
    let mut views: Vec<SeasonalRolloutProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_rollout_proposal_view(item));
    }
    views
}

pub fn sum_seasonal_rollout_proposal_view(view: &SeasonalRolloutProposalView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_ratio + (parsed_rotation_days as f64))
}
