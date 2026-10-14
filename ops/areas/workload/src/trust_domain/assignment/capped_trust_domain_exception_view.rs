use super::*;

use acme_kit::text;

/// How screens and exports show a capped trust domain exception.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedTrustDomainExceptionView {
    pub code: String,
    pub label_text: String,
    pub locale_text: String,
    pub starts_at: String,
    pub namespace: String,
    pub minimum_text: String,
    pub archived_text: String,
    pub scope: String,
    pub discount: String,
    pub status_label: String,
}

pub fn to_capped_trust_domain_exception_view(item: &CappedTrustDomainException) -> CappedTrustDomainExceptionView {
    CappedTrustDomainExceptionView {
        code: text::truncate(&item.code, 80),
        label_text: item.label.clone(),
        locale_text: item.locale.clone(),
        starts_at: item.starts_at.to_string(),
        namespace: item.namespace.clone(),
        minimum_text: format!("{:.2}", item.minimum),
        archived_text: format!("{}", item.archived),
        scope: item.scope.clone(),
        discount: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_trust_domain_exception_views(
    items: &[CappedTrustDomainException],
) -> Vec<CappedTrustDomainExceptionView> {
    let mut views: Vec<CappedTrustDomainExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_trust_domain_exception_view(item));
    }
    views
}

pub fn sum_capped_trust_domain_exception_view(view: &CappedTrustDomainExceptionView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_starts_at as f64 + parsed_minimum + parsed_discount)
}
