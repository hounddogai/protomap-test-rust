use super::*;

use acme_kit::text;

/// How screens and exports show a runbook manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookManifestView {
    pub code: String,
    pub label: String,
    pub locale: String,
    pub title_text: String,
    pub created_at: String,
    pub source: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_runbook_manifest_view(item: &RunbookManifest) -> RunbookManifestView {
    RunbookManifestView {
        code: item.code.clone(),
        label: item.label.clone(),
        locale: item.locale.clone(),
        title_text: text::truncate(&item.title, 64),
        created_at: item.created_at.to_string(),
        source: item.source.clone(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_runbook_manifest_views(items: &[RunbookManifest]) -> Vec<RunbookManifestView> {
    let mut views: Vec<RunbookManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_runbook_manifest_view(item));
    }
    views
}

pub fn sum_runbook_manifest_view(view: &RunbookManifestView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_created_at as f64 + parsed_amount)
}
