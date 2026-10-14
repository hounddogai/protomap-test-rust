use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled rollout rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRolloutRollupView {
    pub code: String,
    pub name_text: String,
    pub scope_text: String,
    pub priority_text: String,
    pub attempts: String,
    pub key_id_text: String,
    pub approved_text: String,
    pub unit_text: String,
    pub rotation_days: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_scheduled_rollout_rollup_view(item: &ScheduledRolloutRollup) -> ScheduledRolloutRollupView {
    ScheduledRolloutRollupView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 64),
        scope_text: item.scope.clone(),
        priority_text: item.priority.to_string(),
        attempts: item.attempts.to_string(),
        key_id_text: item.key_id.clone(),
        approved_text: format!("{}", item.approved),
        unit_text: item.unit.clone(),
        rotation_days: item.rotation_days.to_string(),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_rollout_rollup_views(items: &[ScheduledRolloutRollup]) -> Vec<ScheduledRolloutRollupView> {
    let mut views: Vec<ScheduledRolloutRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_rollout_rollup_view(item));
    }
    views
}

pub fn sum_scheduled_rollout_rollup_view(view: &ScheduledRolloutRollupView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_attempts as f64) + (parsed_rotation_days as f64))
}
