use super::*;

/// How screens and exports show a rule proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleProposalView {
    pub code: String,
    pub title: String,
    pub fingerprint: String,
    pub starts_at: String,
    pub segment: String,
    pub cluster: String,
    pub maximum_text: String,
    pub external_ref: String,
    pub status_label: String,
}

pub fn to_rule_proposal_view(item: &RuleProposal) -> RuleProposalView {
    RuleProposalView {
        code: item.code.clone(),
        title: item.title.clone(),
        fingerprint: item.fingerprint.clone(),
        starts_at: item.starts_at.to_string(),
        segment: item.segment.clone(),
        cluster: item.cluster.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        external_ref: item.external_ref.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rule_proposal_views(items: &[RuleProposal]) -> Vec<RuleProposalView> {
    let mut views: Vec<RuleProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_rule_proposal_view(item));
    }
    views
}

pub fn sum_rule_proposal_view(view: &RuleProposalView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_starts_at as f64 + parsed_maximum)
}
