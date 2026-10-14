use super::*;

/// How screens and exports show a usage manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageManifestView {
    pub code: String,
    pub title: String,
    pub country: String,
    pub revision_text: String,
    pub source: String,
    pub rate_text: String,
    pub sequence: String,
    pub label: String,
    pub cluster_text: String,
    pub updated_at: String,
    pub version_text: String,
    pub status_label: String,
}

pub fn to_usage_manifest_view(item: &UsageManifest) -> UsageManifestView {
    UsageManifestView {
        code: item.code.clone(),
        title: item.title.clone(),
        country: item.country.clone(),
        revision_text: item.revision.to_string(),
        source: item.source.clone(),
        rate_text: format!("{:.2}", item.rate),
        sequence: item.sequence.to_string(),
        label: item.label.clone(),
        cluster_text: item.cluster.clone(),
        updated_at: item.updated_at.to_string(),
        version_text: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_manifest_views(items: &[UsageManifest]) -> Vec<UsageManifestView> {
    let mut views: Vec<UsageManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_manifest_view(item));
    }
    views
}

pub fn sum_usage_manifest_view(view: &UsageManifestView) -> Result<f64, String> {
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_revision as f64 + parsed_rate + (parsed_sequence as f64))
}
