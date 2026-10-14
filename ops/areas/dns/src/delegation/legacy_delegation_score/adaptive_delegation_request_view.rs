use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive delegation request.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDelegationRequestView {
    pub code: String,
    pub name: String,
    pub ceiling: String,
    pub margin_text: String,
    pub updated_at: String,
    pub ttl_seconds: String,
    pub archived: String,
    pub discount: String,
    pub grace_days: String,
    pub cost_text: String,
    pub status_label: String,
}

pub fn to_adaptive_delegation_request_view(item: &AdaptiveDelegationRequest) -> AdaptiveDelegationRequestView {
    AdaptiveDelegationRequestView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 120),
        ceiling: format!("{:.2}", item.ceiling),
        margin_text: format!("{:.2}", item.margin),
        updated_at: item.updated_at.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        archived: format!("{}", item.archived),
        discount: format!("{:.2}", item.discount),
        grace_days: item.grace_days.to_string(),
        cost_text: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_delegation_request_views(items: &[AdaptiveDelegationRequest]) -> Vec<AdaptiveDelegationRequestView> {
    let mut views: Vec<AdaptiveDelegationRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_delegation_request_view(item));
    }
    views
}

pub fn sum_adaptive_delegation_request_view(view: &AdaptiveDelegationRequestView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_ceiling + parsed_margin + (parsed_updated_at as f64))
}
