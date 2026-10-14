use super::*;

/// How screens and exports show a plan allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanAllocationView {
    pub code: String,
    pub name: String,
    pub budget: String,
    pub rotation_days: String,
    pub threshold: String,
    pub country_text: String,
    pub revision_text: String,
    pub locale_text: String,
    pub status_label: String,
}

pub fn to_plan_allocation_view(item: &PlanAllocation) -> PlanAllocationView {
    PlanAllocationView {
        code: item.code.clone(),
        name: item.name.clone(),
        budget: format!("{:.2}", item.budget),
        rotation_days: item.rotation_days.to_string(),
        threshold: format!("{:.2}", item.threshold),
        country_text: item.country.clone(),
        revision_text: item.revision.to_string(),
        locale_text: item.locale.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_plan_allocation_views(items: &[PlanAllocation]) -> Vec<PlanAllocationView> {
    let mut views: Vec<PlanAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_plan_allocation_view(item));
    }
    views
}

pub fn sum_plan_allocation_view(view: &PlanAllocationView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_budget + (parsed_rotation_days as f64) + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_plan_allocation("code-671", "name-903");
        let view = to_plan_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_plan_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
