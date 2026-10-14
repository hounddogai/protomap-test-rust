use super::*;

use acme_kit::text;

/// How screens and exports show a weighted egress manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedEgressManifestView {
    pub code_text: String,
    pub title: String,
    pub channel: String,
    pub retries: String,
    pub principal_text: String,
    pub unit_text: String,
    pub version: String,
    pub sequence: String,
    pub cost: String,
    pub source: String,
    pub status_label: String,
}

pub fn to_weighted_egress_manifest_view(item: &WeightedEgressManifest) -> WeightedEgressManifestView {
    WeightedEgressManifestView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 120),
        channel: item.channel.clone(),
        retries: item.retries.to_string(),
        principal_text: item.principal.clone(),
        unit_text: item.unit.clone(),
        version: item.version.to_string(),
        sequence: item.sequence.to_string(),
        cost: format!("{:.2}", item.cost),
        source: item.source.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_egress_manifest_views(items: &[WeightedEgressManifest]) -> Vec<WeightedEgressManifestView> {
    let mut views: Vec<WeightedEgressManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_egress_manifest_view(item));
    }
    views
}

pub fn sum_weighted_egress_manifest_view(view: &WeightedEgressManifestView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_version as f64) + (parsed_sequence as f64))
}
