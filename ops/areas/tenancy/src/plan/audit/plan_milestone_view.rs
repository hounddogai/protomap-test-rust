use super::*;

use acme_kit::text;

/// How screens and exports show a plan milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanMilestoneView {
    pub code: String,
    pub label: String,
    pub unit_text: String,
    pub locale: String,
    pub ends_at_text: String,
    pub summary: String,
    pub status_label: String,
}

pub fn to_plan_milestone_view(item: &PlanMilestone) -> PlanMilestoneView {
    PlanMilestoneView {
        code: item.code.clone(),
        label: item.label.clone(),
        unit_text: item.unit.clone(),
        locale: text::truncate(&item.locale, 40),
        ends_at_text: item.ends_at.to_string(),
        summary: item.summary.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_plan_milestone_views(items: &[PlanMilestone]) -> Vec<PlanMilestoneView> {
    let mut views: Vec<PlanMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_plan_milestone_view(item));
    }
    views
}

pub fn sum_plan_milestone_view(view: &PlanMilestoneView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_ends_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_plan_milestone("code-841", "label-335");
        let view = to_plan_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_plan_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
