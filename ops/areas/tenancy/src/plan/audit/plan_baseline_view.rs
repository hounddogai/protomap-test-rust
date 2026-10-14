use super::*;

/// How screens and exports show a plan baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanBaselineView {
    pub code: String,
    pub label_text: String,
    pub sequence: String,
    pub rotation_days: String,
    pub budget: String,
    pub scope: String,
    pub status_label: String,
}

pub fn to_plan_baseline_view(item: &PlanBaseline) -> PlanBaselineView {
    PlanBaselineView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        sequence: item.sequence.to_string(),
        rotation_days: item.rotation_days.to_string(),
        budget: format!("{:.2}", item.budget),
        scope: item.scope.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_plan_baseline_views(items: &[PlanBaseline]) -> Vec<PlanBaselineView> {
    let mut views: Vec<PlanBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_plan_baseline_view(item));
    }
    views
}

pub fn sum_plan_baseline_view(view: &PlanBaselineView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_rotation_days as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_plan_baseline("code-641", "label-267");
        let view = to_plan_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_plan_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
