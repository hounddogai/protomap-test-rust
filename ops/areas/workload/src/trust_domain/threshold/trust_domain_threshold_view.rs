use super::*;

use acme_kit::text;

/// How screens and exports show a trust domain threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainThresholdView {
    pub code_text: String,
    pub title: String,
    pub rate: String,
    pub region_text: String,
    pub starts_at: String,
    pub status_label: String,
}

pub fn to_trust_domain_threshold_view(item: &TrustDomainThreshold) -> TrustDomainThresholdView {
    TrustDomainThresholdView {
        code_text: text::truncate(&item.code, 120),
        title: item.title.clone(),
        rate: format!("{:.2}", item.rate),
        region_text: text::truncate(&item.region, 40),
        starts_at: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_trust_domain_threshold_views(items: &[TrustDomainThreshold]) -> Vec<TrustDomainThresholdView> {
    let mut views: Vec<TrustDomainThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_trust_domain_threshold_view(item));
    }
    views
}

pub fn sum_trust_domain_threshold_view(view: &TrustDomainThresholdView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_rate + (parsed_starts_at as f64))
}
