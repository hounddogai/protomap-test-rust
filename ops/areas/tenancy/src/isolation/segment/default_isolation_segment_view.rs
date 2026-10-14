use super::*;

use acme_kit::text;

/// How screens and exports show a default isolation segment.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIsolationSegmentView {
    pub code: String,
    pub label: String,
    pub weight: String,
    pub priority: String,
    pub checksum: String,
    pub fingerprint: String,
    pub burst: String,
    pub scope_text: String,
    pub rate: String,
    pub starts_at: String,
    pub status_label: String,
}

pub fn to_default_isolation_segment_view(item: &DefaultIsolationSegment) -> DefaultIsolationSegmentView {
    DefaultIsolationSegmentView {
        code: text::truncate(&item.code, 80),
        label: text::truncate(&item.label, 80),
        weight: format!("{:.2}", item.weight),
        priority: item.priority.to_string(),
        checksum: item.checksum.clone(),
        fingerprint: item.fingerprint.clone(),
        burst: item.burst.to_string(),
        scope_text: item.scope.clone(),
        rate: format!("{:.2}", item.rate),
        starts_at: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_isolation_segment_views(items: &[DefaultIsolationSegment]) -> Vec<DefaultIsolationSegmentView> {
    let mut views: Vec<DefaultIsolationSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_isolation_segment_view(item));
    }
    views
}

pub fn sum_default_isolation_segment_view(view: &DefaultIsolationSegmentView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_weight + (parsed_priority as f64) + (parsed_burst as f64))
}
