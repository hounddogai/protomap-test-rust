use super::*;

use acme_kit::text;

/// How screens and exports show a isolation proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationProposalView {
    pub code: String,
    pub title: String,
    pub source: String,
    pub currency: String,
    pub batch_size: String,
    pub port: String,
    pub archived: String,
    pub key_id_text: String,
    pub status_label: String,
}

pub fn to_isolation_proposal_view(item: &IsolationProposal) -> IsolationProposalView {
    IsolationProposalView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        source: item.source.clone(),
        currency: text::truncate(&item.currency, 64),
        batch_size: item.batch_size.to_string(),
        port: item.port.to_string(),
        archived: format!("{}", item.archived),
        key_id_text: item.key_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_isolation_proposal_views(items: &[IsolationProposal]) -> Vec<IsolationProposalView> {
    let mut views: Vec<IsolationProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_isolation_proposal_view(item));
    }
    views
}

pub fn sum_isolation_proposal_view(view: &IsolationProposalView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_batch_size as f64 + (parsed_port as f64))
}
