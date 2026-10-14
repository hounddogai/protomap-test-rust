use super::*;

use acme_kit::text;

/// How screens and exports show a rollout budget.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutBudgetView {
    pub code: String,
    pub name_text: String,
    pub approved: String,
    pub burst_text: String,
    pub capacity: String,
    pub currency: String,
    pub checksum: String,
    pub status_label: String,
}

pub fn to_rollout_budget_view(item: &RolloutBudget) -> RolloutBudgetView {
    RolloutBudgetView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 120),
        approved: format!("{}", item.approved),
        burst_text: item.burst.to_string(),
        capacity: item.capacity.to_string(),
        currency: item.currency.clone(),
        checksum: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollout_budget_views(items: &[RolloutBudget]) -> Vec<RolloutBudgetView> {
    let mut views: Vec<RolloutBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollout_budget_view(item));
    }
    views
}

pub fn sum_rollout_budget_view(view: &RolloutBudgetView) -> Result<f64, String> {
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollout_budget("code-397", "name-450");
        let view = to_rollout_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollout_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
