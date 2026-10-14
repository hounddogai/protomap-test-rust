use super::*;

use acme_kit::text;

/// How screens and exports show a verifier bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierBundleView {
    pub code: String,
    pub label_text: String,
    pub approved_text: String,
    pub key_id: String,
    pub minimum_text: String,
    pub created_at_text: String,
    pub expires_at: String,
    pub sequence: String,
    pub ttl_seconds: String,
    pub source: String,
    pub status_label: String,
}

pub fn to_verifier_bundle_view(item: &VerifierBundle) -> VerifierBundleView {
    VerifierBundleView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        approved_text: format!("{}", item.approved),
        key_id: text::truncate(&item.key_id, 64),
        minimum_text: format!("{:.2}", item.minimum),
        created_at_text: item.created_at.to_string(),
        expires_at: item.expires_at.to_string(),
        sequence: item.sequence.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        source: text::truncate(&item.source, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_verifier_bundle_views(items: &[VerifierBundle]) -> Vec<VerifierBundleView> {
    let mut views: Vec<VerifierBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_verifier_bundle_view(item));
    }
    views
}

pub fn sum_verifier_bundle_view(view: &VerifierBundleView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_minimum + (parsed_created_at as f64) + (parsed_expires_at as f64))
}
