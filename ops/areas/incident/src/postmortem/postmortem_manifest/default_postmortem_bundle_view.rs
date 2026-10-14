use super::*;

use acme_kit::text;

/// How screens and exports show a default postmortem bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPostmortemBundleView {
    pub code: String,
    pub name: String,
    pub audience: String,
    pub active: String,
    pub fingerprint: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_default_postmortem_bundle_view(item: &DefaultPostmortemBundle) -> DefaultPostmortemBundleView {
    DefaultPostmortemBundleView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 40),
        audience: item.audience.clone(),
        active: format!("{}", item.active),
        fingerprint: text::truncate(&item.fingerprint, 40),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_postmortem_bundle_views(items: &[DefaultPostmortemBundle]) -> Vec<DefaultPostmortemBundleView> {
    let mut views: Vec<DefaultPostmortemBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_postmortem_bundle_view(item));
    }
    views
}

pub fn sum_default_postmortem_bundle_view(view: &DefaultPostmortemBundleView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}
