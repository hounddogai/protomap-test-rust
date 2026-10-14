use super::*;

use acme_kit::text;

/// How screens and exports show a primary plan assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPlanAssignmentView {
    pub code: String,
    pub label: String,
    pub namespace: String,
    pub priority: String,
    pub budget: String,
    pub quota_units: String,
    pub audience_text: String,
    pub approved_text: String,
    pub channel: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_primary_plan_assignment_view(item: &PrimaryPlanAssignment) -> PrimaryPlanAssignmentView {
    PrimaryPlanAssignmentView {
        code: item.code.clone(),
        label: item.label.clone(),
        namespace: item.namespace.clone(),
        priority: item.priority.to_string(),
        budget: format!("{:.2}", item.budget),
        quota_units: item.quota_units.to_string(),
        audience_text: text::truncate(&item.audience, 40),
        approved_text: format!("{}", item.approved),
        channel: item.channel.clone(),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_plan_assignment_views(items: &[PrimaryPlanAssignment]) -> Vec<PrimaryPlanAssignmentView> {
    let mut views: Vec<PrimaryPlanAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_plan_assignment_view(item));
    }
    views
}

pub fn sum_primary_plan_assignment_view(view: &PrimaryPlanAssignmentView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_priority as f64 + parsed_budget + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_plan_assignment("code-383", "label-985");
        let view = to_primary_plan_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_plan_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
