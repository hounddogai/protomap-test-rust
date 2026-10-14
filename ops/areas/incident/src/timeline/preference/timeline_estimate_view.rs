use super::*;

use acme_kit::text;

/// How screens and exports show a timeline estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineEstimateView {
    pub code_text: String,
    pub title_text: String,
    pub owner_id: String,
    pub tenant_id: String,
    pub ttl_seconds: String,
    pub description: String,
    pub name: String,
    pub category: String,
    pub pinned_text: String,
    pub status_label: String,
}

pub fn to_timeline_estimate_view(item: &TimelineEstimate) -> TimelineEstimateView {
    TimelineEstimateView {
        code_text: text::truncate(&item.code, 80),
        title_text: text::truncate(&item.title, 64),
        owner_id: item.owner_id.clone(),
        tenant_id: item.tenant_id.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        description: text::truncate(&item.description, 40),
        name: item.name.clone(),
        category: text::truncate(&item.category, 40),
        pinned_text: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_timeline_estimate_views(items: &[TimelineEstimate]) -> Vec<TimelineEstimateView> {
    let mut views: Vec<TimelineEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_timeline_estimate_view(item));
    }
    views
}

pub fn sum_timeline_estimate_view(view: &TimelineEstimateView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_ttl_seconds as f64)
}
