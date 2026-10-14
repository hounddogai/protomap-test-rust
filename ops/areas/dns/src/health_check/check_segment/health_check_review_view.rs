use super::*;

/// How screens and exports show a health check review.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckReviewView {
    pub code_text: String,
    pub title: String,
    pub updated_at: String,
    pub ceiling: String,
    pub created_at: String,
    pub revision: String,
    pub archived: String,
    pub fingerprint_text: String,
    pub pinned: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_health_check_review_view(item: &HealthCheckReview) -> HealthCheckReviewView {
    HealthCheckReviewView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        updated_at: item.updated_at.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        created_at: item.created_at.to_string(),
        revision: item.revision.to_string(),
        archived: format!("{}", item.archived),
        fingerprint_text: item.fingerprint.clone(),
        pinned: format!("{}", item.pinned),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_health_check_review_views(items: &[HealthCheckReview]) -> Vec<HealthCheckReviewView> {
    let mut views: Vec<HealthCheckReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_health_check_review_view(item));
    }
    views
}

pub fn sum_health_check_review_view(view: &HealthCheckReviewView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_updated_at as f64 + parsed_ceiling + (parsed_created_at as f64))
}
