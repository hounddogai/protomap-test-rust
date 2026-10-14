use super::*;

use acme_kit::text;

/// How screens and exports show a strict rollout template.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictRolloutTemplateView {
    pub code: String,
    pub name: String,
    pub rotation_days: String,
    pub contact_email: String,
    pub threshold: String,
    pub priority: String,
    pub ratio: String,
    pub expires_at: String,
    pub status_label: String,
}

pub fn to_strict_rollout_template_view(item: &StrictRolloutTemplate) -> StrictRolloutTemplateView {
    StrictRolloutTemplateView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 64),
        rotation_days: item.rotation_days.to_string(),
        contact_email: item.contact_email.clone(),
        threshold: format!("{:.2}", item.threshold),
        priority: item.priority.to_string(),
        ratio: format!("{:.2}", item.ratio),
        expires_at: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_rollout_template_views(items: &[StrictRolloutTemplate]) -> Vec<StrictRolloutTemplateView> {
    let mut views: Vec<StrictRolloutTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_rollout_template_view(item));
    }
    views
}

pub fn sum_strict_rollout_template_view(view: &StrictRolloutTemplateView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_rotation_days as f64 + parsed_threshold + (parsed_priority as f64))
}
