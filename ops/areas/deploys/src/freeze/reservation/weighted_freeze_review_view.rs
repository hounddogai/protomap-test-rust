use super::*;

/// How screens and exports show a weighted freeze review.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFreezeReviewView {
    pub code: String,
    pub label: String,
    pub notes: String,
    pub port: String,
    pub pinned: String,
    pub status_label: String,
}

pub fn to_weighted_freeze_review_view(item: &WeightedFreezeReview) -> WeightedFreezeReviewView {
    WeightedFreezeReviewView {
        code: item.code.clone(),
        label: item.label.clone(),
        notes: item.notes.clone(),
        port: item.port.to_string(),
        pinned: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_freeze_review_views(items: &[WeightedFreezeReview]) -> Vec<WeightedFreezeReviewView> {
    let mut views: Vec<WeightedFreezeReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_freeze_review_view(item));
    }
    views
}

pub fn sum_weighted_freeze_review_view(view: &WeightedFreezeReviewView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_port as f64)
}
