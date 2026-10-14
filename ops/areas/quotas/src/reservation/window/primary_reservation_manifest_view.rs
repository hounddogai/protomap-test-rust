use super::*;

use acme_kit::text;

/// How screens and exports show a primary reservation manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationManifestView {
    pub code: String,
    pub title: String,
    pub source: String,
    pub ceiling: String,
    pub starts_at: String,
    pub active_text: String,
    pub version_text: String,
    pub threshold: String,
    pub issuer: String,
    pub floor_value: String,
    pub limit_text: String,
    pub status_label: String,
}

pub fn to_primary_reservation_manifest_view(item: &PrimaryReservationManifest) -> PrimaryReservationManifestView {
    PrimaryReservationManifestView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 40),
        source: item.source.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        starts_at: item.starts_at.to_string(),
        active_text: format!("{}", item.active),
        version_text: item.version.to_string(),
        threshold: format!("{:.2}", item.threshold),
        issuer: item.issuer.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        limit_text: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_reservation_manifest_views(
    items: &[PrimaryReservationManifest],
) -> Vec<PrimaryReservationManifestView> {
    let mut views: Vec<PrimaryReservationManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_reservation_manifest_view(item));
    }
    views
}

pub fn sum_primary_reservation_manifest_view(view: &PrimaryReservationManifestView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_ceiling + (parsed_starts_at as f64) + (parsed_version as f64))
}
