use super::*;

use acme_kit::text;

/// How screens and exports show a verifier review.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierReviewView {
    pub code_text: String,
    pub name_text: String,
    pub quota_units: String,
    pub cluster_text: String,
    pub ttl_seconds: String,
    pub cost: String,
    pub grace_days: String,
    pub contact_email: String,
    pub revision: String,
    pub status_label: String,
}

pub fn to_verifier_review_view(item: &VerifierReview) -> VerifierReviewView {
    VerifierReviewView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        quota_units: item.quota_units.to_string(),
        cluster_text: item.cluster.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        cost: format!("{:.2}", item.cost),
        grace_days: item.grace_days.to_string(),
        contact_email: text::truncate(&item.contact_email, 120),
        revision: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_verifier_review_views(items: &[VerifierReview]) -> Vec<VerifierReviewView> {
    let mut views: Vec<VerifierReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_verifier_review_view(item));
    }
    views
}

pub fn sum_verifier_review_view(view: &VerifierReviewView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_ttl_seconds as f64) + parsed_cost)
}
