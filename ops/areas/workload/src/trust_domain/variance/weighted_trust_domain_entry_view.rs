use super::*;

use acme_kit::text;

/// How screens and exports show a weighted trust domain entry.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTrustDomainEntryView {
    pub code: String,
    pub label: String,
    pub budget: String,
    pub attempts: String,
    pub ratio: String,
    pub ttl_seconds: String,
    pub status_label: String,
}

pub fn to_weighted_trust_domain_entry_view(item: &WeightedTrustDomainEntry) -> WeightedTrustDomainEntryView {
    WeightedTrustDomainEntryView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 64),
        budget: format!("{:.2}", item.budget),
        attempts: item.attempts.to_string(),
        ratio: format!("{:.2}", item.ratio),
        ttl_seconds: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_trust_domain_entry_views(items: &[WeightedTrustDomainEntry]) -> Vec<WeightedTrustDomainEntryView> {
    let mut views: Vec<WeightedTrustDomainEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_trust_domain_entry_view(item));
    }
    views
}

pub fn sum_weighted_trust_domain_entry_view(view: &WeightedTrustDomainEntryView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_budget + (parsed_attempts as f64) + parsed_ratio)
}
