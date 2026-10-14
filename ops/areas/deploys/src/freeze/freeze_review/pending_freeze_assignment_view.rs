use super::*;

/// How screens and exports show a pending freeze assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingFreezeAssignmentView {
    pub code_text: String,
    pub label: String,
    pub retries: String,
    pub notes: String,
    pub priority_text: String,
    pub created_at: String,
    pub cluster_text: String,
    pub grace_days: String,
    pub title: String,
    pub status_label: String,
}

pub fn to_pending_freeze_assignment_view(item: &PendingFreezeAssignment) -> PendingFreezeAssignmentView {
    PendingFreezeAssignmentView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        retries: item.retries.to_string(),
        notes: item.notes.clone(),
        priority_text: item.priority.to_string(),
        created_at: item.created_at.to_string(),
        cluster_text: item.cluster.clone(),
        grace_days: item.grace_days.to_string(),
        title: item.title.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_freeze_assignment_views(items: &[PendingFreezeAssignment]) -> Vec<PendingFreezeAssignmentView> {
    let mut views: Vec<PendingFreezeAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_freeze_assignment_view(item));
    }
    views
}

pub fn sum_pending_freeze_assignment_view(view: &PendingFreezeAssignmentView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_priority as f64) + (parsed_created_at as f64))
}
