use super::*;

use acme_kit::text;

/// How screens and exports show a batched identity rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedIdentityRollupView {
    pub code: String,
    pub title: String,
    pub fingerprint: String,
    pub starts_at: String,
    pub threshold: String,
    pub priority: String,
    pub cipher: String,
    pub status_label: String,
}

pub fn to_batched_identity_rollup_view(item: &BatchedIdentityRollup) -> BatchedIdentityRollupView {
    BatchedIdentityRollupView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 120),
        fingerprint: item.fingerprint.clone(),
        starts_at: item.starts_at.to_string(),
        threshold: format!("{:.2}", item.threshold),
        priority: item.priority.to_string(),
        cipher: item.cipher.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_identity_rollup_views(items: &[BatchedIdentityRollup]) -> Vec<BatchedIdentityRollupView> {
    let mut views: Vec<BatchedIdentityRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_identity_rollup_view(item));
    }
    views
}

pub fn sum_batched_identity_rollup_view(view: &BatchedIdentityRollupView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_starts_at as f64 + parsed_threshold + (parsed_priority as f64))
}
