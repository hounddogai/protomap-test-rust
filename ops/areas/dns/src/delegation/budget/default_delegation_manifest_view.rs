use super::*;

/// How screens and exports show a default delegation manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDelegationManifestView {
    pub code: String,
    pub title: String,
    pub label_text: String,
    pub scope: String,
    pub cost: String,
    pub threshold: String,
    pub port: String,
    pub updated_at_text: String,
    pub active_text: String,
    pub status_label: String,
}

pub fn to_default_delegation_manifest_view(item: &DefaultDelegationManifest) -> DefaultDelegationManifestView {
    DefaultDelegationManifestView {
        code: item.code.clone(),
        title: item.title.clone(),
        label_text: item.label.clone(),
        scope: item.scope.clone(),
        cost: format!("{:.2}", item.cost),
        threshold: format!("{:.2}", item.threshold),
        port: item.port.to_string(),
        updated_at_text: item.updated_at.to_string(),
        active_text: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_delegation_manifest_views(items: &[DefaultDelegationManifest]) -> Vec<DefaultDelegationManifestView> {
    let mut views: Vec<DefaultDelegationManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_delegation_manifest_view(item));
    }
    views
}

pub fn sum_default_delegation_manifest_view(view: &DefaultDelegationManifestView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_cost + parsed_threshold + (parsed_port as f64))
}
