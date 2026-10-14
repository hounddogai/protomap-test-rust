use super::*;

use acme_kit::text;

/// How screens and exports show a limit proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitProposalView {
    pub code: String,
    pub title: String,
    pub cluster: String,
    pub timezone: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_limit_proposal_view(item: &LimitProposal) -> LimitProposalView {
    LimitProposalView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 80),
        cluster: item.cluster.clone(),
        timezone: text::truncate(&item.timezone, 64),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_limit_proposal_views(items: &[LimitProposal]) -> Vec<LimitProposalView> {
    let mut views: Vec<LimitProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_limit_proposal_view(item));
    }
    views
}

pub fn sum_limit_proposal_view(view: &LimitProposalView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}
