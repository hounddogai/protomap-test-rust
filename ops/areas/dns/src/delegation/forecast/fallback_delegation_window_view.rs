use super::*;

use acme_kit::text;

/// How screens and exports show a fallback delegation window.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDelegationWindowView {
    pub code_text: String,
    pub label: String,
    pub audience: String,
    pub score: String,
    pub tenant_id: String,
    pub description: String,
    pub budget: String,
    pub contact_email: String,
    pub retries: String,
    pub summary: String,
    pub status_label: String,
}

pub fn to_fallback_delegation_window_view(item: &FallbackDelegationWindow) -> FallbackDelegationWindowView {
    FallbackDelegationWindowView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 80),
        audience: text::truncate(&item.audience, 80),
        score: format!("{:.2}", item.score),
        tenant_id: item.tenant_id.clone(),
        description: text::truncate(&item.description, 40),
        budget: format!("{:.2}", item.budget),
        contact_email: text::truncate(&item.contact_email, 40),
        retries: item.retries.to_string(),
        summary: item.summary.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_delegation_window_views(items: &[FallbackDelegationWindow]) -> Vec<FallbackDelegationWindowView> {
    let mut views: Vec<FallbackDelegationWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_delegation_window_view(item));
    }
    views
}

pub fn sum_fallback_delegation_window_view(view: &FallbackDelegationWindowView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_score + parsed_budget + (parsed_retries as f64))
}
