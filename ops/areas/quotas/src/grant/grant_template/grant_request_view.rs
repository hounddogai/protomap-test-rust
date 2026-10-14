use super::*;

use acme_kit::text;

/// How screens and exports show a grant request.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantRequestView {
    pub code: String,
    pub label_text: String,
    pub currency: String,
    pub score: String,
    pub issuer: String,
    pub name: String,
    pub ends_at_text: String,
    pub ttl_seconds: String,
    pub priority_text: String,
    pub country: String,
    pub scope_text: String,
    pub status_label: String,
}

pub fn to_grant_request_view(item: &GrantRequest) -> GrantRequestView {
    GrantRequestView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        currency: text::truncate(&item.currency, 64),
        score: format!("{:.2}", item.score),
        issuer: item.issuer.clone(),
        name: text::truncate(&item.name, 64),
        ends_at_text: item.ends_at.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        priority_text: item.priority.to_string(),
        country: text::truncate(&item.country, 64),
        scope_text: item.scope.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_request_views(items: &[GrantRequest]) -> Vec<GrantRequestView> {
    let mut views: Vec<GrantRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_request_view(item));
    }
    views
}

pub fn sum_grant_request_view(view: &GrantRequestView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_score + (parsed_ends_at as f64) + (parsed_ttl_seconds as f64))
}
