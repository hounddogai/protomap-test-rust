use super::*;

use acme_kit::text;

/// How screens and exports show a global federation target.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFederationTargetView {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub unit_text: String,
    pub minimum: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_global_federation_target_view(item: &GlobalFederationTarget) -> GlobalFederationTargetView {
    GlobalFederationTargetView {
        code: item.code.clone(),
        name: item.name.clone(),
        tenant_id: text::truncate(&item.tenant_id, 64),
        unit_text: item.unit.clone(),
        minimum: format!("{:.2}", item.minimum),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_federation_target_views(items: &[GlobalFederationTarget]) -> Vec<GlobalFederationTargetView> {
    let mut views: Vec<GlobalFederationTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_federation_target_view(item));
    }
    views
}

pub fn sum_global_federation_target_view(view: &GlobalFederationTargetView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_minimum + parsed_maximum)
}
