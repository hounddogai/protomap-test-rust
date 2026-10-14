use super::*;

/// How screens and exports show a capped slo assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedSloAssignmentView {
    pub code_text: String,
    pub title: String,
    pub limit_text: String,
    pub created_at: String,
    pub approved_text: String,
    pub ceiling_text: String,
    pub status_label: String,
}

pub fn to_capped_slo_assignment_view(item: &CappedSloAssignment) -> CappedSloAssignmentView {
    CappedSloAssignmentView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        limit_text: item.limit.to_string(),
        created_at: item.created_at.to_string(),
        approved_text: format!("{}", item.approved),
        ceiling_text: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_slo_assignment_views(items: &[CappedSloAssignment]) -> Vec<CappedSloAssignmentView> {
    let mut views: Vec<CappedSloAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_slo_assignment_view(item));
    }
    views
}

pub fn sum_capped_slo_assignment_view(view: &CappedSloAssignmentView) -> Result<f64, String> {
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_created_at as f64) + parsed_ceiling)
}
