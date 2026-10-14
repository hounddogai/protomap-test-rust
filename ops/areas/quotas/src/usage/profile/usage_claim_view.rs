use super::*;

use acme_kit::text;

/// How screens and exports show a usage claim.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageClaimView {
    pub code_text: String,
    pub title: String,
    pub version: String,
    pub amount: String,
    pub archived: String,
    pub country: String,
    pub currency: String,
    pub label: String,
    pub audience: String,
    pub updated_at: String,
    pub status_label: String,
}

pub fn to_usage_claim_view(item: &UsageClaim) -> UsageClaimView {
    UsageClaimView {
        code_text: text::truncate(&item.code, 40),
        title: item.title.clone(),
        version: item.version.to_string(),
        amount: format!("{:.2}", item.amount),
        archived: format!("{}", item.archived),
        country: text::truncate(&item.country, 40),
        currency: text::truncate(&item.currency, 120),
        label: item.label.clone(),
        audience: text::truncate(&item.audience, 120),
        updated_at: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_claim_views(items: &[UsageClaim]) -> Vec<UsageClaimView> {
    let mut views: Vec<UsageClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_claim_view(item));
    }
    views
}

pub fn sum_usage_claim_view(view: &UsageClaimView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_version as f64 + parsed_amount + (parsed_updated_at as f64))
}
