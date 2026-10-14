use super::*;

use acme_kit::text;

/// How screens and exports show a binding policy.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingPolicyView {
    pub code_text: String,
    pub title: String,
    pub reviewer: String,
    pub priority: String,
    pub floor_value: String,
    pub window_days: String,
    pub attempts: String,
    pub category: String,
    pub status_label: String,
}

pub fn to_binding_policy_view(item: &BindingPolicy) -> BindingPolicyView {
    BindingPolicyView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 40),
        reviewer: text::truncate(&item.reviewer, 64),
        priority: item.priority.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        window_days: item.window_days.to_string(),
        attempts: item.attempts.to_string(),
        category: item.category.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_binding_policy_views(items: &[BindingPolicy]) -> Vec<BindingPolicyView> {
    let mut views: Vec<BindingPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_binding_policy_view(item));
    }
    views
}

pub fn sum_binding_policy_view(view: &BindingPolicyView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_priority as f64 + parsed_floor_value + (parsed_window_days as f64))
}
