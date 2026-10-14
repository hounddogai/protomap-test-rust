use super::*;

use acme_kit::text;

/// How screens and exports show a delegation review.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationReviewView {
    pub code_text: String,
    pub label_text: String,
    pub key_id: String,
    pub discount: String,
    pub phone_text: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_delegation_review_view(item: &DelegationReview) -> DelegationReviewView {
    DelegationReviewView {
        code_text: text::truncate(&item.code, 80),
        label_text: item.label.clone(),
        key_id: item.key_id.clone(),
        discount: format!("{:.2}", item.discount),
        phone_text: item.phone.clone(),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_delegation_review_views(items: &[DelegationReview]) -> Vec<DelegationReviewView> {
    let mut views: Vec<DelegationReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_delegation_review_view(item));
    }
    views
}

pub fn sum_delegation_review_view(view: &DelegationReviewView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_discount + (parsed_quota_units as f64))
}
