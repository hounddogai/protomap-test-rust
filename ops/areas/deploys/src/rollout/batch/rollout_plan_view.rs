use super::*;

use acme_kit::text;

/// How screens and exports show a rollout plan.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutPlanView {
    pub code: String,
    pub title: String,
    pub grace_days: String,
    pub source: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_rollout_plan_view(item: &RolloutPlan) -> RolloutPlanView {
    RolloutPlanView {
        code: item.code.clone(),
        title: item.title.clone(),
        grace_days: item.grace_days.to_string(),
        source: text::truncate(&item.source, 64),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollout_plan_views(items: &[RolloutPlan]) -> Vec<RolloutPlanView> {
    let mut views: Vec<RolloutPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollout_plan_view(item));
    }
    views
}

pub fn sum_rollout_plan_view(view: &RolloutPlanView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_grace_days as f64 + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollout_plan("code-73", "title-900");
        let view = to_rollout_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollout_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
