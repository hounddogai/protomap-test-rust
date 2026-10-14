use super::*;

/// How screens and exports show a primary token review.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTokenReviewView {
    pub code_text: String,
    pub label: String,
    pub ratio: String,
    pub retries: String,
    pub floor_value: String,
    pub category: String,
    pub status_label: String,
}

pub fn to_primary_token_review_view(item: &PrimaryTokenReview) -> PrimaryTokenReviewView {
    PrimaryTokenReviewView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        ratio: format!("{:.2}", item.ratio),
        retries: item.retries.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        category: item.category.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_token_review_views(items: &[PrimaryTokenReview]) -> Vec<PrimaryTokenReviewView> {
    let mut views: Vec<PrimaryTokenReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_token_review_view(item));
    }
    views
}

pub fn sum_primary_token_review_view(view: &PrimaryTokenReviewView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_ratio + (parsed_retries as f64) + parsed_floor_value)
}
