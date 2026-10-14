use super::*;

use acme_kit::text;

/// How screens and exports show a regional federation adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFederationAdjustmentView {
    pub code: String,
    pub label: String,
    pub issuer: String,
    pub reviewer: String,
    pub ends_at: String,
    pub tenant_id_text: String,
    pub revision: String,
    pub status_label: String,
}

pub fn to_regional_federation_adjustment_view(item: &RegionalFederationAdjustment) -> RegionalFederationAdjustmentView {
    RegionalFederationAdjustmentView {
        code: item.code.clone(),
        label: item.label.clone(),
        issuer: item.issuer.clone(),
        reviewer: item.reviewer.clone(),
        ends_at: item.ends_at.to_string(),
        tenant_id_text: text::truncate(&item.tenant_id, 40),
        revision: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_federation_adjustment_views(
    items: &[RegionalFederationAdjustment],
) -> Vec<RegionalFederationAdjustmentView> {
    let mut views: Vec<RegionalFederationAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_federation_adjustment_view(item));
    }
    views
}

pub fn sum_regional_federation_adjustment_view(view: &RegionalFederationAdjustmentView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_revision as f64))
}
