use super::*;

use acme_kit::text;

/// How screens and exports show a canary plan.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryPlanView {
    pub code: String,
    pub name_text: String,
    pub audience_text: String,
    pub minimum: String,
    pub title: String,
    pub status_label: String,
}

pub fn to_canary_plan_view(item: &CanaryPlan) -> CanaryPlanView {
    CanaryPlanView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        audience_text: text::truncate(&item.audience, 120),
        minimum: format!("{:.2}", item.minimum),
        title: item.title.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_canary_plan_views(items: &[CanaryPlan]) -> Vec<CanaryPlanView> {
    let mut views: Vec<CanaryPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_canary_plan_view(item));
    }
    views
}

pub fn sum_canary_plan_view(view: &CanaryPlanView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_canary_plan("code-439", "name-919");
        let view = to_canary_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_canary_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
