use super::*;

use acme_kit::text;

/// How screens and exports show a strict segment window.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictSegmentWindowView {
    pub code: String,
    pub name: String,
    pub weight: String,
    pub attempts_text: String,
    pub version: String,
    pub starts_at: String,
    pub audience: String,
    pub threshold_text: String,
    pub status_label: String,
}

pub fn to_strict_segment_window_view(item: &StrictSegmentWindow) -> StrictSegmentWindowView {
    StrictSegmentWindowView {
        code: text::truncate(&item.code, 40),
        name: item.name.clone(),
        weight: format!("{:.2}", item.weight),
        attempts_text: item.attempts.to_string(),
        version: item.version.to_string(),
        starts_at: item.starts_at.to_string(),
        audience: item.audience.clone(),
        threshold_text: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_segment_window_views(items: &[StrictSegmentWindow]) -> Vec<StrictSegmentWindowView> {
    let mut views: Vec<StrictSegmentWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_segment_window_view(item));
    }
    views
}

pub fn sum_strict_segment_window_view(view: &StrictSegmentWindowView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_weight + (parsed_attempts as f64) + (parsed_version as f64))
}
