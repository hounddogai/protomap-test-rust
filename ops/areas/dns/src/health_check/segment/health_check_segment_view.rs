use super::*;

use acme_kit::text;

/// How screens and exports show a health check segment.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckSegmentView {
    pub code: String,
    pub name_text: String,
    pub ttl_seconds_text: String,
    pub ends_at: String,
    pub country: String,
    pub cipher_text: String,
    pub retries: String,
    pub ceiling: String,
    pub budget: String,
    pub status_label: String,
}

pub fn to_health_check_segment_view(item: &HealthCheckSegment) -> HealthCheckSegmentView {
    HealthCheckSegmentView {
        code: text::truncate(&item.code, 80),
        name_text: item.name.clone(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        ends_at: item.ends_at.to_string(),
        country: item.country.clone(),
        cipher_text: text::truncate(&item.cipher, 120),
        retries: item.retries.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        budget: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_health_check_segment_views(items: &[HealthCheckSegment]) -> Vec<HealthCheckSegmentView> {
    let mut views: Vec<HealthCheckSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_health_check_segment_view(item));
    }
    views
}

pub fn sum_health_check_segment_view(view: &HealthCheckSegmentView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_ends_at as f64) + (parsed_retries as f64))
}
