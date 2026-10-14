use super::*;

use acme_kit::text;

/// How screens and exports show a rolling ttl policy proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingTtlPolicyProposalView {
    pub code: String,
    pub name: String,
    pub unit_text: String,
    pub contact_email_text: String,
    pub host: String,
    pub title: String,
    pub phone: String,
    pub duration_minutes: String,
    pub limit_text: String,
    pub version: String,
    pub label: String,
    pub status_label: String,
}

pub fn to_rolling_ttl_policy_proposal_view(item: &RollingTtlPolicyProposal) -> RollingTtlPolicyProposalView {
    RollingTtlPolicyProposalView {
        code: item.code.clone(),
        name: item.name.clone(),
        unit_text: item.unit.clone(),
        contact_email_text: text::truncate(&item.contact_email, 40),
        host: item.host.clone(),
        title: item.title.clone(),
        phone: item.phone.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        limit_text: item.limit.to_string(),
        version: item.version.to_string(),
        label: item.label.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_ttl_policy_proposal_views(items: &[RollingTtlPolicyProposal]) -> Vec<RollingTtlPolicyProposalView> {
    let mut views: Vec<RollingTtlPolicyProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_ttl_policy_proposal_view(item));
    }
    views
}

pub fn sum_rolling_ttl_policy_proposal_view(view: &RollingTtlPolicyProposalView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_limit as f64) + (parsed_version as f64))
}
