use super::*;

use acme_kit::text;

/// How screens and exports show a lenient ttl policy proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientTtlPolicyProposalView {
    pub code_text: String,
    pub label: String,
    pub archived: String,
    pub batch_size: String,
    pub namespace_text: String,
    pub approved: String,
    pub status_label: String,
}

pub fn to_lenient_ttl_policy_proposal_view(item: &LenientTtlPolicyProposal) -> LenientTtlPolicyProposalView {
    LenientTtlPolicyProposalView {
        code_text: text::truncate(&item.code, 80),
        label: text::truncate(&item.label, 40),
        archived: format!("{}", item.archived),
        batch_size: item.batch_size.to_string(),
        namespace_text: item.namespace.clone(),
        approved: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_ttl_policy_proposal_views(items: &[LenientTtlPolicyProposal]) -> Vec<LenientTtlPolicyProposalView> {
    let mut views: Vec<LenientTtlPolicyProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_ttl_policy_proposal_view(item));
    }
    views
}

pub fn sum_lenient_ttl_policy_proposal_view(view: &LenientTtlPolicyProposalView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_batch_size as f64)
}
