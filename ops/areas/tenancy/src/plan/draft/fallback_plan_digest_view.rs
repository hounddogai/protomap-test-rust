use super::*;

/// How screens and exports show a fallback plan digest.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPlanDigestView {
    pub code: String,
    pub name: String,
    pub currency_text: String,
    pub category: String,
    pub limit: String,
    pub created_at_text: String,
    pub description: String,
    pub discount: String,
    pub quota_units: String,
    pub grace_days: String,
    pub country: String,
    pub status_label: String,
}

pub fn to_fallback_plan_digest_view(item: &FallbackPlanDigest) -> FallbackPlanDigestView {
    FallbackPlanDigestView {
        code: item.code.clone(),
        name: item.name.clone(),
        currency_text: item.currency.clone(),
        category: item.category.clone(),
        limit: item.limit.to_string(),
        created_at_text: item.created_at.to_string(),
        description: item.description.clone(),
        discount: format!("{:.2}", item.discount),
        quota_units: item.quota_units.to_string(),
        grace_days: item.grace_days.to_string(),
        country: item.country.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_plan_digest_views(items: &[FallbackPlanDigest]) -> Vec<FallbackPlanDigestView> {
    let mut views: Vec<FallbackPlanDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_plan_digest_view(item));
    }
    views
}

pub fn sum_fallback_plan_digest_view(view: &FallbackPlanDigestView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_created_at as f64) + parsed_discount)
}
