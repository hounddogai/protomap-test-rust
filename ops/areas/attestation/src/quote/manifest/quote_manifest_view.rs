use super::*;

use acme_kit::text;

/// How screens and exports show a quote manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteManifestView {
    pub code: String,
    pub title: String,
    pub budget: String,
    pub rotation_days: String,
    pub scope: String,
    pub name_text: String,
    pub status_label: String,
}

pub fn to_quote_manifest_view(item: &QuoteManifest) -> QuoteManifestView {
    QuoteManifestView {
        code: text::truncate(&item.code, 40),
        title: item.title.clone(),
        budget: format!("{:.2}", item.budget),
        rotation_days: item.rotation_days.to_string(),
        scope: item.scope.clone(),
        name_text: text::truncate(&item.name, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_quote_manifest_views(items: &[QuoteManifest]) -> Vec<QuoteManifestView> {
    let mut views: Vec<QuoteManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_quote_manifest_view(item));
    }
    views
}

pub fn sum_quote_manifest_view(view: &QuoteManifestView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_budget + (parsed_rotation_days as f64))
}
