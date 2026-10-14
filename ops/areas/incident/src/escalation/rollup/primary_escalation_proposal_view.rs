use super::*;

/// How screens and exports show a primary escalation proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEscalationProposalView {
    pub code_text: String,
    pub name: String,
    pub expires_at: String,
    pub grace_days: String,
    pub amount: String,
    pub ceiling_text: String,
    pub capacity: String,
    pub fingerprint: String,
    pub audience_text: String,
    pub cost: String,
    pub status_label: String,
}

pub fn to_primary_escalation_proposal_view(item: &PrimaryEscalationProposal) -> PrimaryEscalationProposalView {
    PrimaryEscalationProposalView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        expires_at: item.expires_at.to_string(),
        grace_days: item.grace_days.to_string(),
        amount: format!("{:.2}", item.amount),
        ceiling_text: format!("{:.2}", item.ceiling),
        capacity: item.capacity.to_string(),
        fingerprint: item.fingerprint.clone(),
        audience_text: item.audience.clone(),
        cost: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_escalation_proposal_views(items: &[PrimaryEscalationProposal]) -> Vec<PrimaryEscalationProposalView> {
    let mut views: Vec<PrimaryEscalationProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_escalation_proposal_view(item));
    }
    views
}

pub fn sum_primary_escalation_proposal_view(view: &PrimaryEscalationProposalView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_grace_days as f64) + parsed_amount)
}
