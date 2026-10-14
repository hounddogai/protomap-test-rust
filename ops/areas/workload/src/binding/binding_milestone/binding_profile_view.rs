use super::*;

use acme_kit::text;

/// How screens and exports show a binding profile.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingProfileView {
    pub code: String,
    pub label: String,
    pub description: String,
    pub region: String,
    pub active_text: String,
    pub rotation_days: String,
    pub name_text: String,
    pub audience_text: String,
    pub unit: String,
    pub updated_at: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_binding_profile_view(item: &BindingProfile) -> BindingProfileView {
    BindingProfileView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 120),
        description: item.description.clone(),
        region: item.region.clone(),
        active_text: format!("{}", item.active),
        rotation_days: item.rotation_days.to_string(),
        name_text: item.name.clone(),
        audience_text: text::truncate(&item.audience, 64),
        unit: item.unit.clone(),
        updated_at: item.updated_at.to_string(),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_binding_profile_views(items: &[BindingProfile]) -> Vec<BindingProfileView> {
    let mut views: Vec<BindingProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_binding_profile_view(item));
    }
    views
}

pub fn sum_binding_profile_view(view: &BindingProfileView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_updated_at as f64) + (parsed_retries as f64))
}
