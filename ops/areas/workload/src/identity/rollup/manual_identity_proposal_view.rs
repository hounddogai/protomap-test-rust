use super::*;

use acme_kit::text;

/// How screens and exports show a manual identity proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualIdentityProposalView {
    pub code: String,
    pub label: String,
    pub cluster_text: String,
    pub grace_days: String,
    pub principal: String,
    pub checksum_text: String,
    pub approved: String,
    pub scope_text: String,
    pub version: String,
    pub status_label: String,
}

pub fn to_manual_identity_proposal_view(item: &ManualIdentityProposal) -> ManualIdentityProposalView {
    ManualIdentityProposalView {
        code: text::truncate(&item.code, 80),
        label: text::truncate(&item.label, 64),
        cluster_text: item.cluster.clone(),
        grace_days: item.grace_days.to_string(),
        principal: item.principal.clone(),
        checksum_text: item.checksum.clone(),
        approved: format!("{}", item.approved),
        scope_text: item.scope.clone(),
        version: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_identity_proposal_views(items: &[ManualIdentityProposal]) -> Vec<ManualIdentityProposalView> {
    let mut views: Vec<ManualIdentityProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_identity_proposal_view(item));
    }
    views
}

pub fn sum_manual_identity_proposal_view(view: &ManualIdentityProposalView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_version as f64))
}
