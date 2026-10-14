use super::*;

use acme_kit::text;

/// How screens and exports show a primary rollout mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutMappingView {
    pub code_text: String,
    pub label: String,
    pub rotation_days: String,
    pub ratio: String,
    pub principal: String,
    pub checksum_text: String,
    pub audience: String,
    pub approved: String,
    pub description: String,
    pub visible: String,
    pub status_label: String,
}

pub fn to_primary_rollout_mapping_view(item: &PrimaryRolloutMapping) -> PrimaryRolloutMappingView {
    PrimaryRolloutMappingView {
        code_text: text::truncate(&item.code, 64),
        label: item.label.clone(),
        rotation_days: item.rotation_days.to_string(),
        ratio: format!("{:.2}", item.ratio),
        principal: text::truncate(&item.principal, 64),
        checksum_text: item.checksum.clone(),
        audience: text::truncate(&item.audience, 40),
        approved: format!("{}", item.approved),
        description: item.description.clone(),
        visible: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_rollout_mapping_views(items: &[PrimaryRolloutMapping]) -> Vec<PrimaryRolloutMappingView> {
    let mut views: Vec<PrimaryRolloutMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_rollout_mapping_view(item));
    }
    views
}

pub fn sum_primary_rollout_mapping_view(view: &PrimaryRolloutMappingView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_rotation_days as f64 + parsed_ratio)
}
