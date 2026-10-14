use super::*;

use acme_kit::text;

/// How screens and exports show a legacy limit plan.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyLimitPlanView {
    pub code: String,
    pub name_text: String,
    pub limit: String,
    pub window_days: String,
    pub weight: String,
    pub retries_text: String,
    pub status_label: String,
}

pub fn to_legacy_limit_plan_view(item: &LegacyLimitPlan) -> LegacyLimitPlanView {
    LegacyLimitPlanView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 40),
        limit: item.limit.to_string(),
        window_days: item.window_days.to_string(),
        weight: format!("{:.2}", item.weight),
        retries_text: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_limit_plan_views(items: &[LegacyLimitPlan]) -> Vec<LegacyLimitPlanView> {
    let mut views: Vec<LegacyLimitPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_limit_plan_view(item));
    }
    views
}

pub fn sum_legacy_limit_plan_view(view: &LegacyLimitPlanView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_window_days as f64) + parsed_weight)
}
