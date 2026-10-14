use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive region pin plan.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinPlanView {
    pub code_text: String,
    pub title_text: String,
    pub pinned_text: String,
    pub issuer: String,
    pub category: String,
    pub expires_at: String,
    pub minimum: String,
    pub capacity: String,
    pub key_id: String,
    pub starts_at_text: String,
    pub status_label: String,
}

pub fn to_adaptive_region_pin_plan_view(item: &AdaptiveRegionPinPlan) -> AdaptiveRegionPinPlanView {
    AdaptiveRegionPinPlanView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        pinned_text: format!("{}", item.pinned),
        issuer: text::truncate(&item.issuer, 120),
        category: item.category.clone(),
        expires_at: item.expires_at.to_string(),
        minimum: format!("{:.2}", item.minimum),
        capacity: item.capacity.to_string(),
        key_id: item.key_id.clone(),
        starts_at_text: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_region_pin_plan_views(items: &[AdaptiveRegionPinPlan]) -> Vec<AdaptiveRegionPinPlanView> {
    let mut views: Vec<AdaptiveRegionPinPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_region_pin_plan_view(item));
    }
    views
}

pub fn sum_adaptive_region_pin_plan_view(view: &AdaptiveRegionPinPlanView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_expires_at as f64 + parsed_minimum + (parsed_capacity as f64))
}
