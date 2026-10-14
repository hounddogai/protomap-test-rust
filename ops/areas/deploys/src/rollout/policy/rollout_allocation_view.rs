use super::*;

/// How screens and exports show a rollout allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAllocationView {
    pub code: String,
    pub name_text: String,
    pub attempts: String,
    pub checksum_text: String,
    pub created_at: String,
    pub quota_units_text: String,
    pub category_text: String,
    pub status_label: String,
}

pub fn to_rollout_allocation_view(item: &RolloutAllocation) -> RolloutAllocationView {
    RolloutAllocationView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        attempts: item.attempts.to_string(),
        checksum_text: item.checksum.clone(),
        created_at: item.created_at.to_string(),
        quota_units_text: item.quota_units.to_string(),
        category_text: item.category.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollout_allocation_views(items: &[RolloutAllocation]) -> Vec<RolloutAllocationView> {
    let mut views: Vec<RolloutAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollout_allocation_view(item));
    }
    views
}

pub fn sum_rollout_allocation_view(view: &RolloutAllocationView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_created_at as f64) + (parsed_quota_units as f64))
}
