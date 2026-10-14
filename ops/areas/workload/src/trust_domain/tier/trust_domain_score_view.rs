use super::*;

/// How screens and exports show a trust domain score.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainScoreView {
    pub code: String,
    pub label: String,
    pub maximum: String,
    pub ttl_seconds_text: String,
    pub quantity_text: String,
    pub pinned_text: String,
    pub status_label: String,
}

pub fn to_trust_domain_score_view(item: &TrustDomainScore) -> TrustDomainScoreView {
    TrustDomainScoreView {
        code: item.code.clone(),
        label: item.label.clone(),
        maximum: format!("{:.2}", item.maximum),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        quantity_text: item.quantity.to_string(),
        pinned_text: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_trust_domain_score_views(items: &[TrustDomainScore]) -> Vec<TrustDomainScoreView> {
    let mut views: Vec<TrustDomainScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_trust_domain_score_view(item));
    }
    views
}

pub fn sum_trust_domain_score_view(view: &TrustDomainScoreView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_maximum + (parsed_ttl_seconds as f64) + (parsed_quantity as f64))
}
