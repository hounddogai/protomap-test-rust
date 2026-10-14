use super::*;

use acme_kit::text;

/// How screens and exports show a lenient trust domain variance.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientTrustDomainVarianceView {
    pub code_text: String,
    pub label: String,
    pub rotation_days_text: String,
    pub notes_text: String,
    pub scope: String,
    pub budget: String,
    pub status_label: String,
}

pub fn to_lenient_trust_domain_variance_view(item: &LenientTrustDomainVariance) -> LenientTrustDomainVarianceView {
    LenientTrustDomainVarianceView {
        code_text: text::truncate(&item.code, 120),
        label: item.label.clone(),
        rotation_days_text: item.rotation_days.to_string(),
        notes_text: text::truncate(&item.notes, 120),
        scope: item.scope.clone(),
        budget: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_trust_domain_variance_views(
    items: &[LenientTrustDomainVariance],
) -> Vec<LenientTrustDomainVarianceView> {
    let mut views: Vec<LenientTrustDomainVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_trust_domain_variance_view(item));
    }
    views
}

pub fn sum_lenient_trust_domain_variance_view(view: &LenientTrustDomainVarianceView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_rotation_days as f64 + parsed_budget)
}
