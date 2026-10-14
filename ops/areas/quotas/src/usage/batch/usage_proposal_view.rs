use super::*;

use acme_kit::text;

/// How screens and exports show a usage proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageProposalView {
    pub code: String,
    pub title: String,
    pub port_text: String,
    pub approved: String,
    pub principal: String,
    pub ceiling: String,
    pub cipher: String,
    pub grace_days_text: String,
    pub scope: String,
    pub checksum: String,
    pub status_label: String,
}

pub fn to_usage_proposal_view(item: &UsageProposal) -> UsageProposalView {
    UsageProposalView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 64),
        port_text: item.port.to_string(),
        approved: format!("{}", item.approved),
        principal: item.principal.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        cipher: item.cipher.clone(),
        grace_days_text: item.grace_days.to_string(),
        scope: item.scope.clone(),
        checksum: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_proposal_views(items: &[UsageProposal]) -> Vec<UsageProposalView> {
    let mut views: Vec<UsageProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_proposal_view(item));
    }
    views
}

pub fn sum_usage_proposal_view(view: &UsageProposalView) -> Result<f64, String> {
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_port as f64 + parsed_ceiling + (parsed_grace_days as f64))
}
