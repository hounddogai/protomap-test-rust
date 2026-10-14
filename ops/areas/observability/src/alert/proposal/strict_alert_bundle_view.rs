use super::*;

use acme_kit::text;

/// How screens and exports show a strict alert bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictAlertBundleView {
    pub code_text: String,
    pub name_text: String,
    pub starts_at: String,
    pub approved: String,
    pub attempts: String,
    pub cipher: String,
    pub batch_size: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_strict_alert_bundle_view(item: &StrictAlertBundle) -> StrictAlertBundleView {
    StrictAlertBundleView {
        code_text: item.code.clone(),
        name_text: text::truncate(&item.name, 64),
        starts_at: item.starts_at.to_string(),
        approved: format!("{}", item.approved),
        attempts: item.attempts.to_string(),
        cipher: item.cipher.clone(),
        batch_size: item.batch_size.to_string(),
        audience: item.audience.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_alert_bundle_views(items: &[StrictAlertBundle]) -> Vec<StrictAlertBundleView> {
    let mut views: Vec<StrictAlertBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_alert_bundle_view(item));
    }
    views
}

pub fn sum_strict_alert_bundle_view(view: &StrictAlertBundleView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_starts_at as f64 + (parsed_attempts as f64) + (parsed_batch_size as f64))
}
