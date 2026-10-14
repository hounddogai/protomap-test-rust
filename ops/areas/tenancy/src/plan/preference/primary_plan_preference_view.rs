use super::*;

use acme_kit::text;

/// How screens and exports show a primary plan preference.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPlanPreferenceView {
    pub code: String,
    pub name_text: String,
    pub quantity: String,
    pub updated_at: String,
    pub description: String,
    pub status_label: String,
}

pub fn to_primary_plan_preference_view(item: &PrimaryPlanPreference) -> PrimaryPlanPreferenceView {
    PrimaryPlanPreferenceView {
        code: text::truncate(&item.code, 40),
        name_text: text::truncate(&item.name, 64),
        quantity: item.quantity.to_string(),
        updated_at: item.updated_at.to_string(),
        description: item.description.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_plan_preference_views(items: &[PrimaryPlanPreference]) -> Vec<PrimaryPlanPreferenceView> {
    let mut views: Vec<PrimaryPlanPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_plan_preference_view(item));
    }
    views
}

pub fn sum_primary_plan_preference_view(view: &PrimaryPlanPreferenceView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_updated_at as f64))
}
