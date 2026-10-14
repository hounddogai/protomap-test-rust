use super::*;

/// How screens and exports show a delegation manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationManifestView {
    pub code: String,
    pub name: String,
    pub summary: String,
    pub quota_units: String,
    pub locale_text: String,
    pub segment: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_delegation_manifest_view(item: &DelegationManifest) -> DelegationManifestView {
    DelegationManifestView {
        code: item.code.clone(),
        name: item.name.clone(),
        summary: item.summary.clone(),
        quota_units: item.quota_units.to_string(),
        locale_text: item.locale.clone(),
        segment: item.segment.clone(),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_delegation_manifest_views(items: &[DelegationManifest]) -> Vec<DelegationManifestView> {
    let mut views: Vec<DelegationManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_delegation_manifest_view(item));
    }
    views
}

pub fn sum_delegation_manifest_view(view: &DelegationManifestView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_port as f64))
}
