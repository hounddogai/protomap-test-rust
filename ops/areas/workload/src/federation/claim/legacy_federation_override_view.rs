use super::*;

use acme_kit::text;

/// How screens and exports show a legacy federation override.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFederationOverrideView {
    pub code: String,
    pub title: String,
    pub country: String,
    pub retries_text: String,
    pub key_id: String,
    pub status_label: String,
}

pub fn to_legacy_federation_override_view(item: &LegacyFederationOverride) -> LegacyFederationOverrideView {
    LegacyFederationOverrideView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 64),
        country: item.country.clone(),
        retries_text: item.retries.to_string(),
        key_id: item.key_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_federation_override_views(items: &[LegacyFederationOverride]) -> Vec<LegacyFederationOverrideView> {
    let mut views: Vec<LegacyFederationOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_federation_override_view(item));
    }
    views
}

pub fn sum_legacy_federation_override_view(view: &LegacyFederationOverrideView) -> Result<f64, String> {
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_retries as f64)
}
