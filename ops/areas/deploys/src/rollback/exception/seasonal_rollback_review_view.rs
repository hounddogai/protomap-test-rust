use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal rollback review.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRollbackReviewView {
    pub code: String,
    pub name: String,
    pub batch_size: String,
    pub label_text: String,
    pub principal: String,
    pub ttl_seconds: String,
    pub expires_at: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_seasonal_rollback_review_view(item: &SeasonalRollbackReview) -> SeasonalRollbackReviewView {
    SeasonalRollbackReviewView {
        code: text::truncate(&item.code, 40),
        name: item.name.clone(),
        batch_size: item.batch_size.to_string(),
        label_text: text::truncate(&item.label, 120),
        principal: item.principal.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        expires_at: item.expires_at.to_string(),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_rollback_review_views(items: &[SeasonalRollbackReview]) -> Vec<SeasonalRollbackReviewView> {
    let mut views: Vec<SeasonalRollbackReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_rollback_review_view(item));
    }
    views
}

pub fn sum_seasonal_rollback_review_view(view: &SeasonalRollbackReviewView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_batch_size as f64 + (parsed_ttl_seconds as f64) + (parsed_expires_at as f64))
}
