use super::*;

/// How screens and exports show a capped budget burn plan.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnPlanView {
    pub code: String,
    pub label: String,
    pub cost: String,
    pub updated_at_text: String,
    pub ttl_seconds: String,
    pub status_label: String,
}

pub fn to_capped_budget_burn_plan_view(item: &CappedBudgetBurnPlan) -> CappedBudgetBurnPlanView {
    CappedBudgetBurnPlanView {
        code: item.code.clone(),
        label: item.label.clone(),
        cost: format!("{:.2}", item.cost),
        updated_at_text: item.updated_at.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_budget_burn_plan_views(items: &[CappedBudgetBurnPlan]) -> Vec<CappedBudgetBurnPlanView> {
    let mut views: Vec<CappedBudgetBurnPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_budget_burn_plan_view(item));
    }
    views
}

pub fn sum_capped_budget_burn_plan_view(view: &CappedBudgetBurnPlanView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_cost + (parsed_updated_at as f64) + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_budget_burn_plan("code-848", "label-721");
        let view = to_capped_budget_burn_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_budget_burn_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
