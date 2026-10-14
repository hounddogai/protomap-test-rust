use super::*;

use acme_kit::text;

/// How screens and exports show a pending retention proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionProposalView {
    pub code: String,
    pub name: String,
    pub contact_email: String,
    pub minimum: String,
    pub rate_text: String,
    pub starts_at: String,
    pub sequence: String,
    pub status_label: String,
}

pub fn to_pending_retention_proposal_view(item: &PendingRetentionProposal) -> PendingRetentionProposalView {
    PendingRetentionProposalView {
        code: item.code.clone(),
        name: item.name.clone(),
        contact_email: text::truncate(&item.contact_email, 80),
        minimum: format!("{:.2}", item.minimum),
        rate_text: format!("{:.2}", item.rate),
        starts_at: item.starts_at.to_string(),
        sequence: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_retention_proposal_views(items: &[PendingRetentionProposal]) -> Vec<PendingRetentionProposalView> {
    let mut views: Vec<PendingRetentionProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_retention_proposal_view(item));
    }
    views
}

pub fn sum_pending_retention_proposal_view(view: &PendingRetentionProposalView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_minimum + parsed_rate + (parsed_starts_at as f64))
}
