use super::*;

use acme_kit::text;

/// How screens and exports show a postmortem review.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemReviewView {
    pub code: String,
    pub title_text: String,
    pub revision: String,
    pub description_text: String,
    pub phone: String,
    pub updated_at: String,
    pub name: String,
    pub key_id: String,
    pub minimum: String,
    pub audience: String,
    pub created_at_text: String,
    pub status_label: String,
}

pub fn to_postmortem_review_view(item: &PostmortemReview) -> PostmortemReviewView {
    PostmortemReviewView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 120),
        revision: item.revision.to_string(),
        description_text: item.description.clone(),
        phone: text::truncate(&item.phone, 120),
        updated_at: item.updated_at.to_string(),
        name: text::truncate(&item.name, 120),
        key_id: item.key_id.clone(),
        minimum: format!("{:.2}", item.minimum),
        audience: item.audience.clone(),
        created_at_text: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_postmortem_review_views(items: &[PostmortemReview]) -> Vec<PostmortemReviewView> {
    let mut views: Vec<PostmortemReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_postmortem_review_view(item));
    }
    views
}

pub fn sum_postmortem_review_view(view: &PostmortemReviewView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_updated_at as f64) + parsed_minimum)
}
