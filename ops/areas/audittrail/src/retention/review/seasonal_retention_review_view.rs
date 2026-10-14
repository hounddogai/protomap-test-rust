use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal retention review.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRetentionReviewView {
    pub code: String,
    pub label: String,
    pub maximum: String,
    pub expires_at_text: String,
    pub external_ref: String,
    pub principal_text: String,
    pub issuer: String,
    pub approved: String,
    pub quota_units: String,
    pub currency: String,
    pub status_label: String,
}

pub fn to_seasonal_retention_review_view(item: &SeasonalRetentionReview) -> SeasonalRetentionReviewView {
    SeasonalRetentionReviewView {
        code: item.code.clone(),
        label: item.label.clone(),
        maximum: format!("{:.2}", item.maximum),
        expires_at_text: item.expires_at.to_string(),
        external_ref: text::truncate(&item.external_ref, 80),
        principal_text: item.principal.clone(),
        issuer: item.issuer.clone(),
        approved: format!("{}", item.approved),
        quota_units: item.quota_units.to_string(),
        currency: item.currency.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_retention_review_views(items: &[SeasonalRetentionReview]) -> Vec<SeasonalRetentionReviewView> {
    let mut views: Vec<SeasonalRetentionReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_retention_review_view(item));
    }
    views
}

pub fn sum_seasonal_retention_review_view(view: &SeasonalRetentionReviewView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_maximum + (parsed_expires_at as f64) + (parsed_quota_units as f64))
}
