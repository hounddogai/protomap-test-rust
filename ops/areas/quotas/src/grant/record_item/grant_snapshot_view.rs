use super::*;

/// How screens and exports show a grant snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantSnapshotView {
    pub code: String,
    pub title: String,
    pub expires_at: String,
    pub active: String,
    pub attempts: String,
    pub unit: String,
    pub revision: String,
    pub status_label: String,
}

pub fn to_grant_snapshot_view(item: &GrantSnapshot) -> GrantSnapshotView {
    GrantSnapshotView {
        code: item.code.clone(),
        title: item.title.clone(),
        expires_at: item.expires_at.to_string(),
        active: format!("{}", item.active),
        attempts: item.attempts.to_string(),
        unit: item.unit.clone(),
        revision: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_snapshot_views(items: &[GrantSnapshot]) -> Vec<GrantSnapshotView> {
    let mut views: Vec<GrantSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_snapshot_view(item));
    }
    views
}

pub fn sum_grant_snapshot_view(view: &GrantSnapshotView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_attempts as f64) + (parsed_revision as f64))
}
