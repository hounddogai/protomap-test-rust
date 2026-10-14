use super::*;

use acme_kit::text;

/// How screens and exports show a reservation proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationProposalView {
    pub code: String,
    pub name_text: String,
    pub region: String,
    pub title: String,
    pub sequence: String,
    pub approved: String,
    pub description: String,
    pub expires_at_text: String,
    pub reviewer_text: String,
    pub starts_at_text: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_reservation_proposal_view(item: &ReservationProposal) -> ReservationProposalView {
    ReservationProposalView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        region: text::truncate(&item.region, 80),
        title: item.title.clone(),
        sequence: item.sequence.to_string(),
        approved: format!("{}", item.approved),
        description: item.description.clone(),
        expires_at_text: item.expires_at.to_string(),
        reviewer_text: item.reviewer.clone(),
        starts_at_text: item.starts_at.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_reservation_proposal_views(items: &[ReservationProposal]) -> Vec<ReservationProposalView> {
    let mut views: Vec<ReservationProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_reservation_proposal_view(item));
    }
    views
}

pub fn sum_reservation_proposal_view(view: &ReservationProposalView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_expires_at as f64) + (parsed_starts_at as f64))
}
