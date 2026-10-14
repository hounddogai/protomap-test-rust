use super::*;

use acme_kit::text;

/// How screens and exports show a nonce snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceSnapshotView {
    pub code: String,
    pub name: String,
    pub updated_at: String,
    pub unit: String,
    pub owner_id: String,
    pub ends_at_text: String,
    pub rate: String,
    pub timezone: String,
    pub source: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_nonce_snapshot_view(item: &NonceSnapshot) -> NonceSnapshotView {
    NonceSnapshotView {
        code: item.code.clone(),
        name: item.name.clone(),
        updated_at: item.updated_at.to_string(),
        unit: item.unit.clone(),
        owner_id: item.owner_id.clone(),
        ends_at_text: item.ends_at.to_string(),
        rate: format!("{:.2}", item.rate),
        timezone: text::truncate(&item.timezone, 120),
        source: text::truncate(&item.source, 80),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_nonce_snapshot_views(items: &[NonceSnapshot]) -> Vec<NonceSnapshotView> {
    let mut views: Vec<NonceSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_nonce_snapshot_view(item));
    }
    views
}

pub fn sum_nonce_snapshot_view(view: &NonceSnapshotView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_ends_at as f64) + parsed_rate)
}
