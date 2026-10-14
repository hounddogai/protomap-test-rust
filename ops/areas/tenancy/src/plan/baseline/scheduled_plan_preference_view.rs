use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled plan preference.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPlanPreferenceView {
    pub code: String,
    pub name: String,
    pub ttl_seconds: String,
    pub quantity_text: String,
    pub title: String,
    pub limit: String,
    pub category_text: String,
    pub retries: String,
    pub floor_value: String,
    pub status_label: String,
}

pub fn to_scheduled_plan_preference_view(item: &ScheduledPlanPreference) -> ScheduledPlanPreferenceView {
    ScheduledPlanPreferenceView {
        code: item.code.clone(),
        name: item.name.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        quantity_text: item.quantity.to_string(),
        title: text::truncate(&item.title, 64),
        limit: item.limit.to_string(),
        category_text: item.category.clone(),
        retries: item.retries.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_plan_preference_views(items: &[ScheduledPlanPreference]) -> Vec<ScheduledPlanPreferenceView> {
    let mut views: Vec<ScheduledPlanPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_plan_preference_view(item));
    }
    views
}

pub fn sum_scheduled_plan_preference_view(view: &ScheduledPlanPreferenceView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_quantity as f64) + (parsed_limit as f64))
}
