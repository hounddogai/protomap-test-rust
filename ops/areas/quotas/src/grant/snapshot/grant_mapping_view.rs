use super::*;

use acme_kit::text;

/// How screens and exports show a grant mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantMappingView {
    pub code: String,
    pub label_text: String,
    pub discount: String,
    pub checksum: String,
    pub tenant_id: String,
    pub audience: String,
    pub country_text: String,
    pub starts_at: String,
    pub region: String,
    pub fingerprint_text: String,
    pub status_label: String,
}

pub fn to_grant_mapping_view(item: &GrantMapping) -> GrantMappingView {
    GrantMappingView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        discount: format!("{:.2}", item.discount),
        checksum: item.checksum.clone(),
        tenant_id: item.tenant_id.clone(),
        audience: item.audience.clone(),
        country_text: item.country.clone(),
        starts_at: item.starts_at.to_string(),
        region: text::truncate(&item.region, 80),
        fingerprint_text: item.fingerprint.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_mapping_views(items: &[GrantMapping]) -> Vec<GrantMappingView> {
    let mut views: Vec<GrantMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_mapping_view(item));
    }
    views
}

pub fn sum_grant_mapping_view(view: &GrantMappingView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_discount + (parsed_starts_at as f64))
}
