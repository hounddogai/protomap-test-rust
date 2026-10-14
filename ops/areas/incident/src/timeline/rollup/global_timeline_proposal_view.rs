use super::*;

use acme_kit::text;

/// How screens and exports show a global timeline proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTimelineProposalView {
    pub code: String,
    pub name: String,
    pub rate: String,
    pub discount: String,
    pub unit: String,
    pub source_text: String,
    pub status_label: String,
}

pub fn to_global_timeline_proposal_view(item: &GlobalTimelineProposal) -> GlobalTimelineProposalView {
    GlobalTimelineProposalView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        rate: format!("{:.2}", item.rate),
        discount: format!("{:.2}", item.discount),
        unit: item.unit.clone(),
        source_text: text::truncate(&item.source, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_timeline_proposal_views(items: &[GlobalTimelineProposal]) -> Vec<GlobalTimelineProposalView> {
    let mut views: Vec<GlobalTimelineProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_timeline_proposal_view(item));
    }
    views
}

pub fn sum_global_timeline_proposal_view(view: &GlobalTimelineProposalView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_rate + parsed_discount)
}
