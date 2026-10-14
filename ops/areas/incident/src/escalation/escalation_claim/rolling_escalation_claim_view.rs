use super::*;

use acme_kit::text;

/// How screens and exports show a rolling escalation claim.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingEscalationClaimView {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub ttl_seconds: String,
    pub issuer: String,
    pub active: String,
    pub approved: String,
    pub ceiling: String,
    pub category_text: String,
    pub currency_text: String,
    pub status_label: String,
}

pub fn to_rolling_escalation_claim_view(item: &RollingEscalationClaim) -> RollingEscalationClaimView {
    RollingEscalationClaimView {
        code: text::truncate(&item.code, 40),
        name: item.name.clone(),
        tenant_id: item.tenant_id.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        issuer: item.issuer.clone(),
        active: format!("{}", item.active),
        approved: format!("{}", item.approved),
        ceiling: format!("{:.2}", item.ceiling),
        category_text: item.category.clone(),
        currency_text: item.currency.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_escalation_claim_views(items: &[RollingEscalationClaim]) -> Vec<RollingEscalationClaimView> {
    let mut views: Vec<RollingEscalationClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_escalation_claim_view(item));
    }
    views
}

pub fn sum_rolling_escalation_claim_view(view: &RollingEscalationClaimView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + parsed_ceiling)
}
