use super::*;

/// How screens and exports show a batched zone snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedZoneSnapshotView {
    pub code: String,
    pub label: String,
    pub threshold_text: String,
    pub attempts_text: String,
    pub updated_at: String,
    pub pinned: String,
    pub fingerprint_text: String,
    pub status_label: String,
}

pub fn to_batched_zone_snapshot_view(item: &BatchedZoneSnapshot) -> BatchedZoneSnapshotView {
    BatchedZoneSnapshotView {
        code: item.code.clone(),
        label: item.label.clone(),
        threshold_text: format!("{:.2}", item.threshold),
        attempts_text: item.attempts.to_string(),
        updated_at: item.updated_at.to_string(),
        pinned: format!("{}", item.pinned),
        fingerprint_text: item.fingerprint.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_zone_snapshot_views(items: &[BatchedZoneSnapshot]) -> Vec<BatchedZoneSnapshotView> {
    let mut views: Vec<BatchedZoneSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_zone_snapshot_view(item));
    }
    views
}

pub fn sum_batched_zone_snapshot_view(view: &BatchedZoneSnapshotView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_threshold + (parsed_attempts as f64) + (parsed_updated_at as f64))
}
