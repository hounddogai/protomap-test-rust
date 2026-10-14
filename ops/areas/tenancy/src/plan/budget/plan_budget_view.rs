use super::*;

use acme_kit::text;

/// How screens and exports show a plan budget.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanBudgetView {
    pub code_text: String,
    pub name: String,
    pub country: String,
    pub amount: String,
    pub summary_text: String,
    pub ceiling: String,
    pub checksum: String,
    pub notes_text: String,
    pub floor_value: String,
    pub margin_text: String,
    pub title_text: String,
    pub status_label: String,
}

pub fn to_plan_budget_view(item: &PlanBudget) -> PlanBudgetView {
    PlanBudgetView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 80),
        country: item.country.clone(),
        amount: format!("{:.2}", item.amount),
        summary_text: item.summary.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        checksum: item.checksum.clone(),
        notes_text: text::truncate(&item.notes, 120),
        floor_value: format!("{:.2}", item.floor_value),
        margin_text: format!("{:.2}", item.margin),
        title_text: text::truncate(&item.title, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_plan_budget_views(items: &[PlanBudget]) -> Vec<PlanBudgetView> {
    let mut views: Vec<PlanBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_plan_budget_view(item));
    }
    views
}

pub fn sum_plan_budget_view(view: &PlanBudgetView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_amount + parsed_ceiling + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_plan_budget("code-699", "name-363");
        let view = to_plan_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_plan_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
