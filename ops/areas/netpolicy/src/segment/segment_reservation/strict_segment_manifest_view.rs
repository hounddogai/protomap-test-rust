use super::*;

use acme_kit::text;

/// How screens and exports show a strict segment manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictSegmentManifestView {
    pub code: String,
    pub label_text: String,
    pub source: String,
    pub starts_at_text: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_strict_segment_manifest_view(item: &StrictSegmentManifest) -> StrictSegmentManifestView {
    StrictSegmentManifestView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 120),
        source: item.source.clone(),
        starts_at_text: item.starts_at.to_string(),
        cluster: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_segment_manifest_views(items: &[StrictSegmentManifest]) -> Vec<StrictSegmentManifestView> {
    let mut views: Vec<StrictSegmentManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_segment_manifest_view(item));
    }
    views
}

pub fn sum_strict_segment_manifest_view(view: &StrictSegmentManifestView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_starts_at as f64)
}
