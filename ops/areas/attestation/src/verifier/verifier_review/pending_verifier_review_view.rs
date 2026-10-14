use super::*;

use acme_kit::text;

/// How screens and exports show a pending verifier review.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingVerifierReviewView {
    pub code: String,
    pub label: String,
    pub fingerprint_text: String,
    pub principal: String,
    pub ttl_seconds: String,
    pub discount: String,
    pub ratio: String,
    pub unit_text: String,
    pub attempts: String,
    pub limit: String,
    pub status_label: String,
}

pub fn to_pending_verifier_review_view(item: &PendingVerifierReview) -> PendingVerifierReviewView {
    PendingVerifierReviewView {
        code: item.code.clone(),
        label: item.label.clone(),
        fingerprint_text: item.fingerprint.clone(),
        principal: item.principal.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        discount: format!("{:.2}", item.discount),
        ratio: format!("{:.2}", item.ratio),
        unit_text: text::truncate(&item.unit, 120),
        attempts: item.attempts.to_string(),
        limit: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_verifier_review_views(items: &[PendingVerifierReview]) -> Vec<PendingVerifierReviewView> {
    let mut views: Vec<PendingVerifierReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_verifier_review_view(item));
    }
    views
}

pub fn sum_pending_verifier_review_view(view: &PendingVerifierReviewView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + parsed_discount + parsed_ratio)
}
