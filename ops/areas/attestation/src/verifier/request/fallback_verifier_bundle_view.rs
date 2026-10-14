use super::*;

use acme_kit::text;

/// How screens and exports show a fallback verifier bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackVerifierBundleView {
    pub code: String,
    pub title: String,
    pub key_id: String,
    pub name: String,
    pub namespace: String,
    pub threshold_text: String,
    pub starts_at: String,
    pub rotation_days_text: String,
    pub ends_at: String,
    pub active: String,
    pub status_label: String,
}

pub fn to_fallback_verifier_bundle_view(item: &FallbackVerifierBundle) -> FallbackVerifierBundleView {
    FallbackVerifierBundleView {
        code: item.code.clone(),
        title: item.title.clone(),
        key_id: item.key_id.clone(),
        name: text::truncate(&item.name, 80),
        namespace: item.namespace.clone(),
        threshold_text: format!("{:.2}", item.threshold),
        starts_at: item.starts_at.to_string(),
        rotation_days_text: item.rotation_days.to_string(),
        ends_at: item.ends_at.to_string(),
        active: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_verifier_bundle_views(items: &[FallbackVerifierBundle]) -> Vec<FallbackVerifierBundleView> {
    let mut views: Vec<FallbackVerifierBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_verifier_bundle_view(item));
    }
    views
}

pub fn sum_fallback_verifier_bundle_view(view: &FallbackVerifierBundleView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_threshold + (parsed_starts_at as f64) + (parsed_rotation_days as f64))
}
