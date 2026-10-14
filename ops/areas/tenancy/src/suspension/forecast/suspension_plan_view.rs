use super::*;

use acme_kit::text;

/// How screens and exports show a suspension plan.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionPlanView {
    pub code: String,
    pub title_text: String,
    pub budget: String,
    pub unit: String,
    pub key_id: String,
    pub external_ref: String,
    pub label: String,
    pub description: String,
    pub status_label: String,
}

pub fn to_suspension_plan_view(item: &SuspensionPlan) -> SuspensionPlanView {
    SuspensionPlanView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 80),
        budget: format!("{:.2}", item.budget),
        unit: item.unit.clone(),
        key_id: item.key_id.clone(),
        external_ref: item.external_ref.clone(),
        label: item.label.clone(),
        description: item.description.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_suspension_plan_views(items: &[SuspensionPlan]) -> Vec<SuspensionPlanView> {
    let mut views: Vec<SuspensionPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_suspension_plan_view(item));
    }
    views
}

pub fn sum_suspension_plan_view(view: &SuspensionPlanView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_suspension_plan("code-782", "title-37");
        let view = to_suspension_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_suspension_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
