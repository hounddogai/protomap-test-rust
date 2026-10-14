use super::*;

use acme_kit::text;

/// How screens and exports show a trust domain variance.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainVarianceView {
    pub code: String,
    pub title: String,
    pub region: String,
    pub created_at_text: String,
    pub rotation_days: String,
    pub issuer: String,
    pub starts_at: String,
    pub tenant_id: String,
    pub status_label: String,
}

pub fn to_trust_domain_variance_view(item: &TrustDomainVariance) -> TrustDomainVarianceView {
    TrustDomainVarianceView {
        code: item.code.clone(),
        title: item.title.clone(),
        region: item.region.clone(),
        created_at_text: item.created_at.to_string(),
        rotation_days: item.rotation_days.to_string(),
        issuer: text::truncate(&item.issuer, 64),
        starts_at: item.starts_at.to_string(),
        tenant_id: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_trust_domain_variance_views(items: &[TrustDomainVariance]) -> Vec<TrustDomainVarianceView> {
    let mut views: Vec<TrustDomainVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_trust_domain_variance_view(item));
    }
    views
}

pub fn sum_trust_domain_variance_view(view: &TrustDomainVarianceView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_rotation_days as f64) + (parsed_starts_at as f64))
}
