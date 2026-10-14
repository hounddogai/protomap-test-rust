use super::*;

use acme_kit::text;

/// How screens and exports show a fallback plan template.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPlanTemplateView {
    pub code: String,
    pub label_text: String,
    pub ttl_seconds: String,
    pub attempts_text: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_fallback_plan_template_view(item: &FallbackPlanTemplate) -> FallbackPlanTemplateView {
    FallbackPlanTemplateView {
        code: text::truncate(&item.code, 120),
        label_text: item.label.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        attempts_text: item.attempts.to_string(),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_plan_template_views(items: &[FallbackPlanTemplate]) -> Vec<FallbackPlanTemplateView> {
    let mut views: Vec<FallbackPlanTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_plan_template_view(item));
    }
    views
}

pub fn sum_fallback_plan_template_view(view: &FallbackPlanTemplateView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_attempts as f64) + (parsed_rotation_days as f64))
}
