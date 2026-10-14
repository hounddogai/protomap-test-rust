use super::*;

use acme_kit::text;

/// How screens and exports show a limit plan.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitPlanView {
    pub code: String,
    pub label_text: String,
    pub description_text: String,
    pub cost_text: String,
    pub name: String,
    pub notes: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_limit_plan_view(item: &LimitPlan) -> LimitPlanView {
    LimitPlanView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 64),
        description_text: text::truncate(&item.description, 64),
        cost_text: format!("{:.2}", item.cost),
        name: item.name.clone(),
        notes: item.notes.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_limit_plan_views(items: &[LimitPlan]) -> Vec<LimitPlanView> {
    let mut views: Vec<LimitPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_limit_plan_view(item));
    }
    views
}

pub fn sum_limit_plan_view(view: &LimitPlanView) -> Result<f64, String> {
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_cost + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_limit_plan("code-119", "label-547");
        let view = to_limit_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_limit_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
