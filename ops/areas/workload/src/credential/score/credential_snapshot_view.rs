use super::*;

use acme_kit::text;

/// How screens and exports show a credential snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialSnapshotView {
    pub code_text: String,
    pub label: String,
    pub rate_text: String,
    pub revision: String,
    pub host: String,
    pub namespace: String,
    pub timezone_text: String,
    pub status_label: String,
}

pub fn to_credential_snapshot_view(item: &CredentialSnapshot) -> CredentialSnapshotView {
    CredentialSnapshotView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 64),
        rate_text: format!("{:.2}", item.rate),
        revision: item.revision.to_string(),
        host: item.host.clone(),
        namespace: text::truncate(&item.namespace, 40),
        timezone_text: text::truncate(&item.timezone, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_credential_snapshot_views(items: &[CredentialSnapshot]) -> Vec<CredentialSnapshotView> {
    let mut views: Vec<CredentialSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_credential_snapshot_view(item));
    }
    views
}

pub fn sum_credential_snapshot_view(view: &CredentialSnapshotView) -> Result<f64, String> {
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_rate + (parsed_revision as f64))
}
