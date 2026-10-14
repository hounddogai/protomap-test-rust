use super::*;

use acme_kit::text;

/// How screens and exports show a capped egress review.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedEgressReviewView {
    pub code: String,
    pub title: String,
    pub host: String,
    pub pinned: String,
    pub quota_units: String,
    pub segment_text: String,
    pub score_text: String,
    pub currency: String,
    pub unit_text: String,
    pub status_label: String,
}

pub fn to_capped_egress_review_view(item: &CappedEgressReview) -> CappedEgressReviewView {
    CappedEgressReviewView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 120),
        host: item.host.clone(),
        pinned: format!("{}", item.pinned),
        quota_units: item.quota_units.to_string(),
        segment_text: text::truncate(&item.segment, 40),
        score_text: format!("{:.2}", item.score),
        currency: item.currency.clone(),
        unit_text: text::truncate(&item.unit, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_egress_review_views(items: &[CappedEgressReview]) -> Vec<CappedEgressReviewView> {
    let mut views: Vec<CappedEgressReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_egress_review_view(item));
    }
    views
}

pub fn sum_capped_egress_review_view(view: &CappedEgressReviewView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_score)
}
