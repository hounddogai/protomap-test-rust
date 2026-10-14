use super::*;

use acme_kit::text;

/// How screens and exports show a batched health check segment.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedHealthCheckSegmentView {
    pub code: String,
    pub label_text: String,
    pub ends_at: String,
    pub attempts_text: String,
    pub checksum_text: String,
    pub host: String,
    pub expires_at_text: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_batched_health_check_segment_view(item: &BatchedHealthCheckSegment) -> BatchedHealthCheckSegmentView {
    BatchedHealthCheckSegmentView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 80),
        ends_at: item.ends_at.to_string(),
        attempts_text: item.attempts.to_string(),
        checksum_text: item.checksum.clone(),
        host: item.host.clone(),
        expires_at_text: item.expires_at.to_string(),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_health_check_segment_views(
    items: &[BatchedHealthCheckSegment],
) -> Vec<BatchedHealthCheckSegmentView> {
    let mut views: Vec<BatchedHealthCheckSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_health_check_segment_view(item));
    }
    views
}

pub fn sum_batched_health_check_segment_view(view: &BatchedHealthCheckSegmentView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_attempts as f64) + (parsed_expires_at as f64))
}
