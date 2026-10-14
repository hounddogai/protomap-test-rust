use super::*;

use acme_kit::text;

/// How screens and exports show a rolling plan assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPlanAssignmentView {
    pub code: String,
    pub title_text: String,
    pub owner_id: String,
    pub weight: String,
    pub principal: String,
    pub host_text: String,
    pub channel: String,
    pub status_label: String,
}

pub fn to_rolling_plan_assignment_view(item: &RollingPlanAssignment) -> RollingPlanAssignmentView {
    RollingPlanAssignmentView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        owner_id: item.owner_id.clone(),
        weight: format!("{:.2}", item.weight),
        principal: text::truncate(&item.principal, 64),
        host_text: text::truncate(&item.host, 40),
        channel: text::truncate(&item.channel, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_plan_assignment_views(items: &[RollingPlanAssignment]) -> Vec<RollingPlanAssignmentView> {
    let mut views: Vec<RollingPlanAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_plan_assignment_view(item));
    }
    views
}

pub fn sum_rolling_plan_assignment_view(view: &RollingPlanAssignmentView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_weight)
}
