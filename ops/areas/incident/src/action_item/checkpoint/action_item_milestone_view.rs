use super::*;

use acme_kit::text;

/// How screens and exports show a action item milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemMilestoneView {
    pub code: String,
    pub label_text: String,
    pub updated_at: String,
    pub capacity: String,
    pub duration_minutes: String,
    pub reviewer: String,
    pub channel: String,
    pub issuer_text: String,
    pub starts_at: String,
    pub external_ref: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_action_item_milestone_view(item: &ActionItemMilestone) -> ActionItemMilestoneView {
    ActionItemMilestoneView {
        code: text::truncate(&item.code, 64),
        label_text: item.label.clone(),
        updated_at: item.updated_at.to_string(),
        capacity: item.capacity.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        reviewer: text::truncate(&item.reviewer, 64),
        channel: item.channel.clone(),
        issuer_text: item.issuer.clone(),
        starts_at: item.starts_at.to_string(),
        external_ref: text::truncate(&item.external_ref, 64),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_action_item_milestone_views(items: &[ActionItemMilestone]) -> Vec<ActionItemMilestoneView> {
    let mut views: Vec<ActionItemMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_action_item_milestone_view(item));
    }
    views
}

pub fn sum_action_item_milestone_view(view: &ActionItemMilestoneView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_capacity as f64) + (parsed_duration_minutes as f64))
}
