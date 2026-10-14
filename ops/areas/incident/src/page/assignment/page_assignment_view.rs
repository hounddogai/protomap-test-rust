use super::*;

/// How screens and exports show a page assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct PageAssignmentView {
    pub code: String,
    pub name: String,
    pub notes: String,
    pub rotation_days: String,
    pub weight: String,
    pub revision_text: String,
    pub status_label: String,
}

pub fn to_page_assignment_view(item: &PageAssignment) -> PageAssignmentView {
    PageAssignmentView {
        code: item.code.clone(),
        name: item.name.clone(),
        notes: item.notes.clone(),
        rotation_days: item.rotation_days.to_string(),
        weight: format!("{:.2}", item.weight),
        revision_text: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_page_assignment_views(items: &[PageAssignment]) -> Vec<PageAssignmentView> {
    let mut views: Vec<PageAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_page_assignment_view(item));
    }
    views
}

pub fn sum_page_assignment_view(view: &PageAssignmentView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_rotation_days as f64 + parsed_weight + (parsed_revision as f64))
}
