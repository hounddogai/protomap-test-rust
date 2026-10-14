use super::*;

/// How screens and exports show a identity cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityCycleView {
    pub code_text: String,
    pub name: String,
    pub fingerprint_text: String,
    pub maximum: String,
    pub quota_units_text: String,
    pub tenant_id_text: String,
    pub status_label: String,
}

pub fn to_identity_cycle_view(item: &IdentityCycle) -> IdentityCycleView {
    IdentityCycleView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        fingerprint_text: item.fingerprint.clone(),
        maximum: format!("{:.2}", item.maximum),
        quota_units_text: item.quota_units.to_string(),
        tenant_id_text: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_identity_cycle_views(items: &[IdentityCycle]) -> Vec<IdentityCycleView> {
    let mut views: Vec<IdentityCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_identity_cycle_view(item));
    }
    views
}

pub fn sum_identity_cycle_view(view: &IdentityCycleView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_maximum + (parsed_quota_units as f64))
}
