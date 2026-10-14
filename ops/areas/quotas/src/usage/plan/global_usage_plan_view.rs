use super::*;

use acme_kit::text;

/// How screens and exports show a global usage plan.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalUsagePlanView {
    pub code: String,
    pub label: String,
    pub ceiling: String,
    pub cost: String,
    pub margin: String,
    pub title_text: String,
    pub status_label: String,
}

pub fn to_global_usage_plan_view(item: &GlobalUsagePlan) -> GlobalUsagePlanView {
    GlobalUsagePlanView {
        code: item.code.clone(),
        label: item.label.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        cost: format!("{:.2}", item.cost),
        margin: format!("{:.2}", item.margin),
        title_text: text::truncate(&item.title, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_usage_plan_views(items: &[GlobalUsagePlan]) -> Vec<GlobalUsagePlanView> {
    let mut views: Vec<GlobalUsagePlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_usage_plan_view(item));
    }
    views
}

pub fn sum_global_usage_plan_view(view: &GlobalUsagePlanView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_ceiling + parsed_cost + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_usage_plan("code-380", "label-346");
        let view = to_global_usage_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_usage_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
