use super::*;

use acme_kit::text;

/// How screens and exports show a usage profile.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageProfileView {
    pub code: String,
    pub label_text: String,
    pub archived: String,
    pub updated_at: String,
    pub ratio: String,
    pub reviewer: String,
    pub tenant_id: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_usage_profile_view(item: &UsageProfile) -> UsageProfileView {
    UsageProfileView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 120),
        archived: format!("{}", item.archived),
        updated_at: item.updated_at.to_string(),
        ratio: format!("{:.2}", item.ratio),
        reviewer: item.reviewer.clone(),
        tenant_id: item.tenant_id.clone(),
        audience: text::truncate(&item.audience, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_profile_views(items: &[UsageProfile]) -> Vec<UsageProfileView> {
    let mut views: Vec<UsageProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_profile_view(item));
    }
    views
}

pub fn sum_usage_profile_view(view: &UsageProfileView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_updated_at as f64 + parsed_ratio)
}
