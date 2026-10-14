use super::*;

use acme_kit::text;

/// How screens and exports show a rolling verifier snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierSnapshotView {
    pub code_text: String,
    pub name: String,
    pub external_ref_text: String,
    pub fingerprint: String,
    pub retries: String,
    pub label: String,
    pub quota_units: String,
    pub maximum: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_rolling_verifier_snapshot_view(item: &RollingVerifierSnapshot) -> RollingVerifierSnapshotView {
    RollingVerifierSnapshotView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        external_ref_text: item.external_ref.clone(),
        fingerprint: text::truncate(&item.fingerprint, 80),
        retries: item.retries.to_string(),
        label: item.label.clone(),
        quota_units: item.quota_units.to_string(),
        maximum: format!("{:.2}", item.maximum),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_verifier_snapshot_views(items: &[RollingVerifierSnapshot]) -> Vec<RollingVerifierSnapshotView> {
    let mut views: Vec<RollingVerifierSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_verifier_snapshot_view(item));
    }
    views
}

pub fn sum_rolling_verifier_snapshot_view(view: &RollingVerifierSnapshotView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_quota_units as f64) + parsed_maximum)
}
