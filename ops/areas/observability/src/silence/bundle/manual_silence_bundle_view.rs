use super::*;

use acme_kit::text;

/// How screens and exports show a manual silence bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualSilenceBundleView {
    pub code_text: String,
    pub label: String,
    pub pinned: String,
    pub limit: String,
    pub approved: String,
    pub timezone_text: String,
    pub issuer_text: String,
    pub namespace: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_manual_silence_bundle_view(item: &ManualSilenceBundle) -> ManualSilenceBundleView {
    ManualSilenceBundleView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        pinned: format!("{}", item.pinned),
        limit: item.limit.to_string(),
        approved: format!("{}", item.approved),
        timezone_text: item.timezone.clone(),
        issuer_text: text::truncate(&item.issuer, 40),
        namespace: text::truncate(&item.namespace, 40),
        principal: item.principal.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_silence_bundle_views(items: &[ManualSilenceBundle]) -> Vec<ManualSilenceBundleView> {
    let mut views: Vec<ManualSilenceBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_silence_bundle_view(item));
    }
    views
}

pub fn sum_manual_silence_bundle_view(view: &ManualSilenceBundleView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_limit as f64)
}
