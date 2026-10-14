use super::*;

use acme_kit::text;

/// How screens and exports show a legacy binding claim.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingClaimView {
    pub code: String,
    pub name: String,
    pub retries: String,
    pub grace_days: String,
    pub ceiling: String,
    pub title_text: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_legacy_binding_claim_view(item: &LegacyBindingClaim) -> LegacyBindingClaimView {
    LegacyBindingClaimView {
        code: text::truncate(&item.code, 40),
        name: item.name.clone(),
        retries: item.retries.to_string(),
        grace_days: item.grace_days.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        title_text: item.title.clone(),
        reviewer: text::truncate(&item.reviewer, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_binding_claim_views(items: &[LegacyBindingClaim]) -> Vec<LegacyBindingClaimView> {
    let mut views: Vec<LegacyBindingClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_binding_claim_view(item));
    }
    views
}

pub fn sum_legacy_binding_claim_view(view: &LegacyBindingClaimView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_grace_days as f64) + parsed_ceiling)
}
