use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled rollout budget.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRolloutBudgetView {
    pub code: String,
    pub label: String,
    pub source: String,
    pub approved: String,
    pub maximum: String,
    pub name: String,
    pub batch_size_text: String,
    pub country: String,
    pub cost: String,
    pub scope: String,
    pub namespace_text: String,
    pub status_label: String,
}

pub fn to_scheduled_rollout_budget_view(item: &ScheduledRolloutBudget) -> ScheduledRolloutBudgetView {
    ScheduledRolloutBudgetView {
        code: item.code.clone(),
        label: item.label.clone(),
        source: item.source.clone(),
        approved: format!("{}", item.approved),
        maximum: format!("{:.2}", item.maximum),
        name: item.name.clone(),
        batch_size_text: item.batch_size.to_string(),
        country: item.country.clone(),
        cost: format!("{:.2}", item.cost),
        scope: text::truncate(&item.scope, 80),
        namespace_text: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_rollout_budget_views(items: &[ScheduledRolloutBudget]) -> Vec<ScheduledRolloutBudgetView> {
    let mut views: Vec<ScheduledRolloutBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_rollout_budget_view(item));
    }
    views
}

pub fn sum_scheduled_rollout_budget_view(view: &ScheduledRolloutBudgetView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_maximum + (parsed_batch_size as f64) + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_rollout_budget("code-680", "label-590");
        let view = to_scheduled_rollout_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_rollout_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
