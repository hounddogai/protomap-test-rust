use super::*;

use acme_kit::text;

/// How screens and exports show a rolling peering plan.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPeeringPlanView {
    pub code_text: String,
    pub label: String,
    pub retries: String,
    pub reviewer_text: String,
    pub region: String,
    pub weight: String,
    pub scope_text: String,
    pub grace_days: String,
    pub locale_text: String,
    pub status_label: String,
}

pub fn to_rolling_peering_plan_view(item: &RollingPeeringPlan) -> RollingPeeringPlanView {
    RollingPeeringPlanView {
        code_text: text::truncate(&item.code, 120),
        label: item.label.clone(),
        retries: item.retries.to_string(),
        reviewer_text: item.reviewer.clone(),
        region: item.region.clone(),
        weight: format!("{:.2}", item.weight),
        scope_text: text::truncate(&item.scope, 64),
        grace_days: item.grace_days.to_string(),
        locale_text: text::truncate(&item.locale, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_peering_plan_views(items: &[RollingPeeringPlan]) -> Vec<RollingPeeringPlanView> {
    let mut views: Vec<RollingPeeringPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_peering_plan_view(item));
    }
    views
}

pub fn sum_rolling_peering_plan_view(view: &RollingPeeringPlanView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_retries as f64 + parsed_weight + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_peering_plan("code-321", "label-241");
        let view = to_rolling_peering_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_peering_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
