use super::*;

use acme_kit::text;

/// How screens and exports show a rollout entry.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutEntryView {
    pub code_text: String,
    pub name: String,
    pub ttl_seconds: String,
    pub priority: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_rollout_entry_view(item: &RolloutEntry) -> RolloutEntryView {
    RolloutEntryView {
        code_text: text::truncate(&item.code, 120),
        name: text::truncate(&item.name, 80),
        ttl_seconds: item.ttl_seconds.to_string(),
        priority: item.priority.to_string(),
        unit: item.unit.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollout_entry_views(items: &[RolloutEntry]) -> Vec<RolloutEntryView> {
    let mut views: Vec<RolloutEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollout_entry_view(item));
    }
    views
}

pub fn sum_rollout_entry_view(view: &RolloutEntryView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_priority as f64))
}
