use super::*;

use acme_kit::text;

/// How screens and exports show a action item assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemAssignmentView {
    pub code: String,
    pub name_text: String,
    pub attempts: String,
    pub grace_days: String,
    pub visible: String,
    pub duration_minutes: String,
    pub margin: String,
    pub minimum_text: String,
    pub checksum: String,
    pub retries_text: String,
    pub status_label: String,
}

pub fn to_action_item_assignment_view(item: &ActionItemAssignment) -> ActionItemAssignmentView {
    ActionItemAssignmentView {
        code: text::truncate(&item.code, 64),
        name_text: text::truncate(&item.name, 40),
        attempts: item.attempts.to_string(),
        grace_days: item.grace_days.to_string(),
        visible: format!("{}", item.visible),
        duration_minutes: item.duration_minutes.to_string(),
        margin: format!("{:.2}", item.margin),
        minimum_text: format!("{:.2}", item.minimum),
        checksum: item.checksum.clone(),
        retries_text: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_action_item_assignment_views(items: &[ActionItemAssignment]) -> Vec<ActionItemAssignmentView> {
    let mut views: Vec<ActionItemAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_action_item_assignment_view(item));
    }
    views
}

pub fn sum_action_item_assignment_view(view: &ActionItemAssignmentView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_grace_days as f64) + (parsed_duration_minutes as f64))
}
