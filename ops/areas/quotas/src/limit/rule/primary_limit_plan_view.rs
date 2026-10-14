use super::*;

use acme_kit::text;

/// How screens and exports show a primary limit plan.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitPlanView {
    pub code: String,
    pub label_text: String,
    pub budget_text: String,
    pub threshold_text: String,
    pub ratio: String,
    pub ttl_seconds_text: String,
    pub region: String,
    pub maximum_text: String,
    pub status_label: String,
}

pub fn to_primary_limit_plan_view(item: &PrimaryLimitPlan) -> PrimaryLimitPlanView {
    PrimaryLimitPlanView {
        code: text::truncate(&item.code, 120),
        label_text: text::truncate(&item.label, 64),
        budget_text: format!("{:.2}", item.budget),
        threshold_text: format!("{:.2}", item.threshold),
        ratio: format!("{:.2}", item.ratio),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        region: item.region.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_limit_plan_views(items: &[PrimaryLimitPlan]) -> Vec<PrimaryLimitPlanView> {
    let mut views: Vec<PrimaryLimitPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_limit_plan_view(item));
    }
    views
}

pub fn sum_primary_limit_plan_view(view: &PrimaryLimitPlanView) -> Result<f64, String> {
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_budget + parsed_threshold + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_limit_plan("code-469", "label-72");
        let view = to_primary_limit_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_limit_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
