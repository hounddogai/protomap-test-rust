use super::*;

use acme_kit::text;

/// How screens and exports show a rollout template.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutTemplateView {
    pub code_text: String,
    pub label: String,
    pub starts_at: String,
    pub locale: String,
    pub window_days: String,
    pub owner_id_text: String,
    pub status_label: String,
}

pub fn to_rollout_template_view(item: &RolloutTemplate) -> RolloutTemplateView {
    RolloutTemplateView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        starts_at: item.starts_at.to_string(),
        locale: item.locale.clone(),
        window_days: item.window_days.to_string(),
        owner_id_text: text::truncate(&item.owner_id, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollout_template_views(items: &[RolloutTemplate]) -> Vec<RolloutTemplateView> {
    let mut views: Vec<RolloutTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollout_template_view(item));
    }
    views
}

pub fn sum_rollout_template_view(view: &RolloutTemplateView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_starts_at as f64 + (parsed_window_days as f64))
}
