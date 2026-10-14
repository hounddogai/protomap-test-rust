use super::*;

use acme_kit::text;

/// How screens and exports show a weighted query manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedQueryManifestView {
    pub code: String,
    pub label: String,
    pub key_id_text: String,
    pub channel: String,
    pub country: String,
    pub sequence_text: String,
    pub created_at: String,
    pub timezone: String,
    pub currency: String,
    pub description_text: String,
    pub status_label: String,
}

pub fn to_weighted_query_manifest_view(item: &WeightedQueryManifest) -> WeightedQueryManifestView {
    WeightedQueryManifestView {
        code: text::truncate(&item.code, 80),
        label: item.label.clone(),
        key_id_text: item.key_id.clone(),
        channel: text::truncate(&item.channel, 64),
        country: item.country.clone(),
        sequence_text: item.sequence.to_string(),
        created_at: item.created_at.to_string(),
        timezone: item.timezone.clone(),
        currency: item.currency.clone(),
        description_text: text::truncate(&item.description, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_query_manifest_views(items: &[WeightedQueryManifest]) -> Vec<WeightedQueryManifestView> {
    let mut views: Vec<WeightedQueryManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_query_manifest_view(item));
    }
    views
}

pub fn sum_weighted_query_manifest_view(view: &WeightedQueryManifestView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_created_at as f64))
}
