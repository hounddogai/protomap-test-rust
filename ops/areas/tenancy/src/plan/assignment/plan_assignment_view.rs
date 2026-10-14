use super::*;

use acme_kit::text;

/// How screens and exports show a plan assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanAssignmentView {
    pub code: String,
    pub title_text: String,
    pub floor_value: String,
    pub quota_units: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_plan_assignment_view(item: &PlanAssignment) -> PlanAssignmentView {
    PlanAssignmentView {
        code: text::truncate(&item.code, 64),
        title_text: item.title.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        quota_units: item.quota_units.to_string(),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_plan_assignment_views(items: &[PlanAssignment]) -> Vec<PlanAssignmentView> {
    let mut views: Vec<PlanAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_plan_assignment_view(item));
    }
    views
}

pub fn sum_plan_assignment_view(view: &PlanAssignmentView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_floor_value + (parsed_quota_units as f64) + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_plan_assignment("code-328", "title-923");
        let view = to_plan_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_plan_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
