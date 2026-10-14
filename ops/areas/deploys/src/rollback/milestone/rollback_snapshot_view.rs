use super::*;

use acme_kit::text;

/// How screens and exports show a rollback snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackSnapshotView {
    pub code: String,
    pub label: String,
    pub audience: String,
    pub minimum_text: String,
    pub source: String,
    pub expires_at: String,
    pub segment: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_rollback_snapshot_view(item: &RollbackSnapshot) -> RollbackSnapshotView {
    RollbackSnapshotView {
        code: text::truncate(&item.code, 120),
        label: item.label.clone(),
        audience: item.audience.clone(),
        minimum_text: format!("{:.2}", item.minimum),
        source: item.source.clone(),
        expires_at: item.expires_at.to_string(),
        segment: item.segment.clone(),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollback_snapshot_views(items: &[RollbackSnapshot]) -> Vec<RollbackSnapshotView> {
    let mut views: Vec<RollbackSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollback_snapshot_view(item));
    }
    views
}

pub fn sum_rollback_snapshot_view(view: &RollbackSnapshotView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_minimum + (parsed_expires_at as f64))
}
