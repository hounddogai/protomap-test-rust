use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive timeline plan.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTimelinePlanView {
    pub code: String,
    pub label: String,
    pub retries: String,
    pub attempts: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_adaptive_timeline_plan_view(item: &AdaptiveTimelinePlan) -> AdaptiveTimelinePlanView {
    AdaptiveTimelinePlanView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 64),
        retries: item.retries.to_string(),
        attempts: item.attempts.to_string(),
        reviewer: item.reviewer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_timeline_plan_views(items: &[AdaptiveTimelinePlan]) -> Vec<AdaptiveTimelinePlanView> {
    let mut views: Vec<AdaptiveTimelinePlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_timeline_plan_view(item));
    }
    views
}

pub fn sum_adaptive_timeline_plan_view(view: &AdaptiveTimelinePlanView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_attempts as f64))
}
