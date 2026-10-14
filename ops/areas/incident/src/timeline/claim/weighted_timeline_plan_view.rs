use super::*;

use acme_kit::text;

/// How screens and exports show a weighted timeline plan.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTimelinePlanView {
    pub code: String,
    pub label_text: String,
    pub approved_text: String,
    pub version: String,
    pub timezone: String,
    pub budget: String,
    pub archived_text: String,
    pub attempts_text: String,
    pub status_label: String,
}

pub fn to_weighted_timeline_plan_view(item: &WeightedTimelinePlan) -> WeightedTimelinePlanView {
    WeightedTimelinePlanView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        approved_text: format!("{}", item.approved),
        version: item.version.to_string(),
        timezone: text::truncate(&item.timezone, 120),
        budget: format!("{:.2}", item.budget),
        archived_text: format!("{}", item.archived),
        attempts_text: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_timeline_plan_views(items: &[WeightedTimelinePlan]) -> Vec<WeightedTimelinePlanView> {
    let mut views: Vec<WeightedTimelinePlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_timeline_plan_view(item));
    }
    views
}

pub fn sum_weighted_timeline_plan_view(view: &WeightedTimelinePlanView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_version as f64 + parsed_budget + (parsed_attempts as f64))
}
