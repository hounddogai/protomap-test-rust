use super::*;

use acme_kit::text;

/// How screens and exports show a rollout assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAssignmentView {
    pub code_text: String,
    pub label: String,
    pub quantity_text: String,
    pub source: String,
    pub channel: String,
    pub namespace_text: String,
    pub audience: String,
    pub external_ref_text: String,
    pub notes: String,
    pub description_text: String,
    pub duration_minutes: String,
    pub status_label: String,
}

pub fn to_rollout_assignment_view(item: &RolloutAssignment) -> RolloutAssignmentView {
    RolloutAssignmentView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        quantity_text: item.quantity.to_string(),
        source: item.source.clone(),
        channel: text::truncate(&item.channel, 40),
        namespace_text: item.namespace.clone(),
        audience: item.audience.clone(),
        external_ref_text: text::truncate(&item.external_ref, 120),
        notes: item.notes.clone(),
        description_text: text::truncate(&item.description, 80),
        duration_minutes: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollout_assignment_views(items: &[RolloutAssignment]) -> Vec<RolloutAssignmentView> {
    let mut views: Vec<RolloutAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollout_assignment_view(item));
    }
    views
}

pub fn sum_rollout_assignment_view(view: &RolloutAssignmentView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_duration_minutes as f64))
}
