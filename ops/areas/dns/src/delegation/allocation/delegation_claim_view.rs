use super::*;

use acme_kit::text;

/// How screens and exports show a delegation claim.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationClaimView {
    pub code: String,
    pub name: String,
    pub approved: String,
    pub ceiling_text: String,
    pub burst: String,
    pub tenant_id: String,
    pub cluster: String,
    pub key_id: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_delegation_claim_view(item: &DelegationClaim) -> DelegationClaimView {
    DelegationClaimView {
        code: item.code.clone(),
        name: item.name.clone(),
        approved: format!("{}", item.approved),
        ceiling_text: format!("{:.2}", item.ceiling),
        burst: item.burst.to_string(),
        tenant_id: text::truncate(&item.tenant_id, 80),
        cluster: item.cluster.clone(),
        key_id: item.key_id.clone(),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_delegation_claim_views(items: &[DelegationClaim]) -> Vec<DelegationClaimView> {
    let mut views: Vec<DelegationClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_delegation_claim_view(item));
    }
    views
}

pub fn sum_delegation_claim_view(view: &DelegationClaimView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_ceiling + (parsed_burst as f64) + (parsed_capacity as f64))
}
