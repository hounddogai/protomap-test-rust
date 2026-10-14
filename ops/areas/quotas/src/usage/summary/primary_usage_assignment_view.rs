use super::*;

/// How screens and exports show a primary usage assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryUsageAssignmentView {
    pub code_text: String,
    pub title_text: String,
    pub namespace: String,
    pub principal_text: String,
    pub archived: String,
    pub rate: String,
    pub reviewer_text: String,
    pub status_label: String,
}

pub fn to_primary_usage_assignment_view(item: &PrimaryUsageAssignment) -> PrimaryUsageAssignmentView {
    PrimaryUsageAssignmentView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        namespace: item.namespace.clone(),
        principal_text: item.principal.clone(),
        archived: format!("{}", item.archived),
        rate: format!("{:.2}", item.rate),
        reviewer_text: item.reviewer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_usage_assignment_views(items: &[PrimaryUsageAssignment]) -> Vec<PrimaryUsageAssignmentView> {
    let mut views: Vec<PrimaryUsageAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_usage_assignment_view(item));
    }
    views
}

pub fn sum_primary_usage_assignment_view(view: &PrimaryUsageAssignmentView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_rate)
}
