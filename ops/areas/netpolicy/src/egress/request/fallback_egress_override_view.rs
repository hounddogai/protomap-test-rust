use super::*;

use acme_kit::text;

/// How screens and exports show a fallback egress override.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEgressOverrideView {
    pub code_text: String,
    pub name: String,
    pub source: String,
    pub cipher: String,
    pub version_text: String,
    pub issuer: String,
    pub status_label: String,
}

pub fn to_fallback_egress_override_view(item: &FallbackEgressOverride) -> FallbackEgressOverrideView {
    FallbackEgressOverrideView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 64),
        source: text::truncate(&item.source, 40),
        cipher: item.cipher.clone(),
        version_text: item.version.to_string(),
        issuer: item.issuer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_egress_override_views(items: &[FallbackEgressOverride]) -> Vec<FallbackEgressOverrideView> {
    let mut views: Vec<FallbackEgressOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_egress_override_view(item));
    }
    views
}

pub fn sum_fallback_egress_override_view(view: &FallbackEgressOverrideView) -> Result<f64, String> {
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_version as f64)
}
