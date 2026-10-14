use super::*;

use acme_kit::text;

/// How screens and exports show a lenient nonce variance.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientNonceVarianceView {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub audience_text: String,
    pub grace_days_text: String,
    pub scope: String,
    pub source: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_lenient_nonce_variance_view(item: &LenientNonceVariance) -> LenientNonceVarianceView {
    LenientNonceVarianceView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 80),
        owner_id: item.owner_id.clone(),
        audience_text: text::truncate(&item.audience, 80),
        grace_days_text: item.grace_days.to_string(),
        scope: item.scope.clone(),
        source: item.source.clone(),
        cluster: text::truncate(&item.cluster, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_nonce_variance_views(items: &[LenientNonceVariance]) -> Vec<LenientNonceVarianceView> {
    let mut views: Vec<LenientNonceVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_nonce_variance_view(item));
    }
    views
}

pub fn sum_lenient_nonce_variance_view(view: &LenientNonceVarianceView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_grace_days as f64)
}
