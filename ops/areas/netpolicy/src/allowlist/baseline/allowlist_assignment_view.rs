use super::*;

use acme_kit::text;

/// How screens and exports show a allowlist assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistAssignmentView {
    pub code: String,
    pub title_text: String,
    pub weight: String,
    pub locale: String,
    pub category_text: String,
    pub segment: String,
    pub status_label: String,
}

pub fn to_allowlist_assignment_view(item: &AllowlistAssignment) -> AllowlistAssignmentView {
    AllowlistAssignmentView {
        code: text::truncate(&item.code, 80),
        title_text: text::truncate(&item.title, 80),
        weight: format!("{:.2}", item.weight),
        locale: item.locale.clone(),
        category_text: text::truncate(&item.category, 120),
        segment: text::truncate(&item.segment, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_allowlist_assignment_views(items: &[AllowlistAssignment]) -> Vec<AllowlistAssignmentView> {
    let mut views: Vec<AllowlistAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_allowlist_assignment_view(item));
    }
    views
}

pub fn sum_allowlist_assignment_view(view: &AllowlistAssignmentView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_weight)
}
