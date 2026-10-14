use super::*;

use acme_kit::text;

/// How screens and exports show a failover review.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverReviewView {
    pub code: String,
    pub name_text: String,
    pub label: String,
    pub window_days: String,
    pub maximum: String,
    pub audience: String,
    pub priority_text: String,
    pub status_label: String,
}

pub fn to_failover_review_view(item: &FailoverReview) -> FailoverReviewView {
    FailoverReviewView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 80),
        label: item.label.clone(),
        window_days: item.window_days.to_string(),
        maximum: format!("{:.2}", item.maximum),
        audience: item.audience.clone(),
        priority_text: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_failover_review_views(items: &[FailoverReview]) -> Vec<FailoverReviewView> {
    let mut views: Vec<FailoverReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_failover_review_view(item));
    }
    views
}

pub fn sum_failover_review_view(view: &FailoverReviewView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_window_days as f64 + parsed_maximum + (parsed_priority as f64))
}
