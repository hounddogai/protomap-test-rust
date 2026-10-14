use super::*;

/// How screens and exports show a default zone review.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultZoneReviewView {
    pub code: String,
    pub name: String,
    pub cost_text: String,
    pub active_text: String,
    pub contact_email: String,
    pub status_label: String,
}

pub fn to_default_zone_review_view(item: &DefaultZoneReview) -> DefaultZoneReviewView {
    DefaultZoneReviewView {
        code: item.code.clone(),
        name: item.name.clone(),
        cost_text: format!("{:.2}", item.cost),
        active_text: format!("{}", item.active),
        contact_email: item.contact_email.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_zone_review_views(items: &[DefaultZoneReview]) -> Vec<DefaultZoneReviewView> {
    let mut views: Vec<DefaultZoneReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_zone_review_view(item));
    }
    views
}

pub fn sum_default_zone_review_view(view: &DefaultZoneReviewView) -> Result<f64, String> {
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_cost)
}
