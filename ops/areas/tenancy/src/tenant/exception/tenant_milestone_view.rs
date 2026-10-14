use super::*;

use acme_kit::text;

/// How screens and exports show a tenant milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantMilestoneView {
    pub code: String,
    pub label: String,
    pub floor_value: String,
    pub cipher_text: String,
    pub fingerprint: String,
    pub title: String,
    pub tenant_id_text: String,
    pub capacity: String,
    pub version: String,
    pub status_label: String,
}

pub fn to_tenant_milestone_view(item: &TenantMilestone) -> TenantMilestoneView {
    TenantMilestoneView {
        code: item.code.clone(),
        label: item.label.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        cipher_text: item.cipher.clone(),
        fingerprint: text::truncate(&item.fingerprint, 40),
        title: item.title.clone(),
        tenant_id_text: item.tenant_id.clone(),
        capacity: item.capacity.to_string(),
        version: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tenant_milestone_views(items: &[TenantMilestone]) -> Vec<TenantMilestoneView> {
    let mut views: Vec<TenantMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_tenant_milestone_view(item));
    }
    views
}

pub fn sum_tenant_milestone_view(view: &TenantMilestoneView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_floor_value + (parsed_capacity as f64) + (parsed_version as f64))
}
