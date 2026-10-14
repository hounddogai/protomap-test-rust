use super::*;

/// How screens and exports show a weighted timeline review.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTimelineReviewView {
    pub code: String,
    pub label: String,
    pub visible: String,
    pub created_at: String,
    pub quantity: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_weighted_timeline_review_view(item: &WeightedTimelineReview) -> WeightedTimelineReviewView {
    WeightedTimelineReviewView {
        code: item.code.clone(),
        label: item.label.clone(),
        visible: format!("{}", item.visible),
        created_at: item.created_at.to_string(),
        quantity: item.quantity.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_timeline_review_views(items: &[WeightedTimelineReview]) -> Vec<WeightedTimelineReviewView> {
    let mut views: Vec<WeightedTimelineReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_timeline_review_view(item));
    }
    views
}

pub fn sum_weighted_timeline_review_view(view: &WeightedTimelineReviewView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_quantity as f64) + parsed_ceiling)
}
