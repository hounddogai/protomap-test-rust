use super::*;

/// How screens and exports show a rollout cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutCycleView {
    pub code_text: String,
    pub label_text: String,
    pub retries: String,
    pub timezone: String,
    pub pinned: String,
    pub limit: String,
    pub capacity: String,
    pub segment: String,
    pub weight: String,
    pub status_label: String,
}

pub fn to_rollout_cycle_view(item: &RolloutCycle) -> RolloutCycleView {
    RolloutCycleView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        retries: item.retries.to_string(),
        timezone: item.timezone.clone(),
        pinned: format!("{}", item.pinned),
        limit: item.limit.to_string(),
        capacity: item.capacity.to_string(),
        segment: item.segment.clone(),
        weight: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollout_cycle_views(items: &[RolloutCycle]) -> Vec<RolloutCycleView> {
    let mut views: Vec<RolloutCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollout_cycle_view(item));
    }
    views
}

pub fn sum_rollout_cycle_view(view: &RolloutCycleView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_limit as f64) + (parsed_capacity as f64))
}
