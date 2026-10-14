use super::*;

use acme_kit::text;

/// How screens and exports show a delegation tier.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationTierView {
    pub code: String,
    pub name: String,
    pub sequence: String,
    pub owner_id: String,
    pub created_at: String,
    pub priority: String,
    pub category_text: String,
    pub external_ref_text: String,
    pub ratio_text: String,
    pub floor_value: String,
    pub description: String,
    pub status_label: String,
}

pub fn to_delegation_tier_view(item: &DelegationTier) -> DelegationTierView {
    DelegationTierView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        sequence: item.sequence.to_string(),
        owner_id: item.owner_id.clone(),
        created_at: item.created_at.to_string(),
        priority: item.priority.to_string(),
        category_text: item.category.clone(),
        external_ref_text: text::truncate(&item.external_ref, 64),
        ratio_text: format!("{:.2}", item.ratio),
        floor_value: format!("{:.2}", item.floor_value),
        description: item.description.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_delegation_tier_views(items: &[DelegationTier]) -> Vec<DelegationTierView> {
    let mut views: Vec<DelegationTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_delegation_tier_view(item));
    }
    views
}

pub fn sum_delegation_tier_view(view: &DelegationTierView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_created_at as f64) + (parsed_priority as f64))
}
