use super::*;

use acme_kit::text;

/// How screens and exports show a default release batch.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseBatchView {
    pub code_text: String,
    pub title: String,
    pub burst: String,
    pub pinned: String,
    pub cluster: String,
    pub unit: String,
    pub duration_minutes: String,
    pub expires_at_text: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_default_release_batch_view(item: &DefaultReleaseBatch) -> DefaultReleaseBatchView {
    DefaultReleaseBatchView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 40),
        burst: item.burst.to_string(),
        pinned: format!("{}", item.pinned),
        cluster: item.cluster.clone(),
        unit: item.unit.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        expires_at_text: item.expires_at.to_string(),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_release_batch_views(items: &[DefaultReleaseBatch]) -> Vec<DefaultReleaseBatchView> {
    let mut views: Vec<DefaultReleaseBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_release_batch_view(item));
    }
    views
}

pub fn sum_default_release_batch_view(view: &DefaultReleaseBatchView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_duration_minutes as f64) + (parsed_expires_at as f64))
}
